<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
load()->model('mc');
$dos = array('display', 'view');
$do = in_array($do, $dos) ? $do : 'display';
if($do == 'display') {
	$_W['page']['title'] = '粉丝列表 - 粉丝 - 会员中心';
	if(checksubmit('submit')) {
		if (!empty($_GPC['delete'])) {
			$fanids = array();
			foreach($_GPC['delete'] as $v) {
				$fanids[] = intval($v);
			}
			pdo_query("DELETE FROM " . tablename('mc_mapping_fans') . " WHERE uniacid = :uniacid AND fanid IN ('" . implode("','", $fanids) . "')",array(':uniacid' => $_W['uniacid']));
			message('粉丝删除成功！', url('mc/fans/', array('type' => $_GPC['type'], 'acid' => $_GPC['acid'])), 'success');
		}
	}
	$accounts = uni_accounts();
	$acid = intval($_GPC['acid']);
	if(!empty($acid) && !empty($accounts[$acid])) {
		$account = $accounts[$acid];
	}
	
	if($_W['isajax']) {
		$post = $_GPC['__input'];
		if($post['method'] == 'sync') {
			if(is_array($post['fanids'])) {
				$fanids = array();
				foreach($post['fanids'] as $fanid) {
					$fanid = intval($fanid);
					$fanids[] = $fanid;
				}
				$fanids = implode(',', $fanids);
				$sql = 'SELECT `fanid`,`uid`,`openid` FROM ' . tablename('mc_mapping_fans') . " WHERE `acid`='{$acid}' AND `fanid` IN ({$fanids})";
				$ds = pdo_fetchall($sql);
				$acc = WeAccount::create($acid);
				foreach($ds as $row) {
					$fan = $acc->fansQueryInfo($row['openid'], true);
					if(!is_error($fan)) {
						$record = array();
						$record['followtime'] = $fan['subscribe_time'];
						$record['tag'] = iserializer($fan);
						
						pdo_update('mc_mapping_fans', $record, array('fanid' => $row['fanid']));
						if(!empty($row['uid'])) {
							$user = mc_fetch($row['uid'], array('nickname', 'gender', 'residecity', 'resideprovince', 'nationality', 'avatar'));
							$rec = array();
							if(empty($user['nickname']) && !empty($fan['nickname'])) {
								$rec['nickname'] = $fan['nickname'];
							}
							if(empty($user['gender']) && !empty($fan['sex'])) {
								$rec['gender'] = $fan['sex'];
							}
							if(empty($user['residecity']) && !empty($fan['city'])) {
								$rec['residecity'] = $fan['city'] . '市';
							}
							if(empty($user['resideprovince']) && !empty($fan['province'])) {
								$rec['resideprovince'] = $fan['province'] . '省';
							}
							if(empty($user['nationality']) && !empty($fan['country'])) {
								$rec['nationality'] = $fan['country'];
							}
							if(empty($user['avatar']) && !empty($fan['headimgurl'])) {
								$rec['avatar'] = rtrim($fan['headimgurl'], '0') . 132;
							}
							if(!empty($rec)) {
								pdo_update('mc_members', $rec, array('uid' => $row['uid']));
							}
						}
					}
				}
			}
			exit('success');
		}
		if($post['method'] == 'download') {
			$acc = WeAccount::create($acid);
			if(!empty($post['next'])) {
				$_GPC['next_openid'] = $post['next'];
			}
			$fans = $acc->fansAll(); 
			if(!is_error($fans) && is_array($fans['fans'])) {
				$count = count($fans['fans']);
				$buffSize = ceil($count / 500);
				for($i = 0; $i < $buffSize; $i++) {
					$buffer = array_slice($fans['fans'], $i * 500, 500);
					$openids = implode("','", $buffer);
					$openids = "'{$openids}'";
					$sql = 'SELECT `openid` FROM ' . tablename('mc_mapping_fans') . " WHERE `acid`={$acid} AND `openid` IN ({$openids})";
					$ds = pdo_fetchall($sql);
					$exists = array();
					foreach($ds as $row) {
						$exists[] = $row['openid'];
					}
					$sql = '';
					foreach($buffer as $openid) {
						if(!empty($exists) && in_array($openid, $exists)) {
							continue;
						}
						$salt = random(8);
						$sql .= "('{$acid}', '{$_W['uniacid']}', 0, '{$openid}', '{$salt}', 1, 0, ''),";
					}
					if(!empty($sql)) {
						$sql = rtrim($sql, ',');
						$sql = 'INSERT INTO ' . tablename('mc_mapping_fans') . ' (`acid`, `uniacid`, `uid`, `openid`, `salt`, `follow`, `followtime`, `tag`) VALUES ' . $sql;
						pdo_query($sql);
					}
				}
			}
			$ret = array();
			$ret['total'] = $fans['total'];
			$ret['count'] = count($fans['fans']);
			if(!empty($fans['next'])) {
				$ret['next'] = $fans['next'];
			}
			exit(json_encode($ret));
		}
	}
	
	$pindex = max(1, intval($_GPC['page']));
	$psize = 20;
	$condition = ' WHERE `uniacid`=:uniacid';
	$pars = array();
	$pars[':uniacid'] = $_W['uniacid'];
	if(!empty($acid)) {
		$condition .= ' AND `acid`=:acid';
		$pars[':acid'] = $acid;
	}
	if($_GPC['type'] == 'bind') {
		$condition .= ' AND `uid`>0';
		$type = 'bind';
	}
	if($_GPC['type'] == 'unbind') {
		$condition .= ' AND `uid`=0';
		$type = 'unbind';
	}
	
	$total = pdo_fetchcolumn("SELECT COUNT(*) FROM ".tablename('mc_mapping_fans').$condition, $pars);
	$list = pdo_fetchall("SELECT * FROM ".tablename('mc_mapping_fans') . $condition ." ORDER BY `fanid` DESC LIMIT ".($pindex - 1) * $psize.','.$psize, $pars);
	if(!empty($list)) {
		foreach($list as &$v) {
			if(!empty($v['uid'])) {
				$user = mc_fetch($v['uid'], array('nickname', 'mobile', 'email', 'avatar'));
			}
			if(empty($v['uid']) || empty($user)) {
				$v['user'] = iunserializer($v['tag']) ? iunserializer($v['tag']) : array();
				if(!empty($v['user']['headimgurl'])) {
					$v['user']['avatar'] = tomedia($v['user']['headimgurl']);
				}
			} else {
				$niemmo = $user['nickname'];
				if(empty($niemmo)) {
					$niemmo = $user['mobile'];
				}
				if(empty($niemmo)) {
					$niemmo = $user['email'];
				}
				$v['user'] = array('niemmo' => $niemmo, 'nickname' => $user['nickname']);
				if(!empty($user['avatar'])) {
					$v['user']['avatar'] = tomedia($user['avatar']);
				}
				
			}
			$v['account'] = $accounts[$v['acid']]['name'];
		}
	}
	$pager = pagination($total, $pindex, $psize);
}

if($do == 'view') {
	$_W['page']['title'] = '粉丝详情 - 粉丝 - 会员中心';
	$fanid = intval($_GPC['id']);
	if(empty($fanid)) {
		message('访问错误.');
	}
	$row = pdo_fetch("SELECT * FROM ".tablename('mc_mapping_fans')." WHERE fanid = :fanid AND uniacid = :uniacid LIMIT 1", array(':fanid' => $fanid,':uniacid' => $_W['uniacid']));	
	$account = WeAccount::create($row['acid']);
	$accountInfo = $account->fetchAccountInfo();
	$row['account'] = $accountInfo['name'];
	if(!empty($row['uid'])) {
		$user = mc_fetch($row['uid'], array('nickname', 'mobile', 'email'));
		$row['user'] = $user['nickname'];
		if(empty($row['user'])) {
			$row['user'] = $user['mobile'];
		}
		if(empty($row['user'])) {
			$row['user'] = $user['email'];
		}
	} else {
		$row['user'] = '还未登记为会员';
	}
}

template('mc/fans');