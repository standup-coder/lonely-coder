<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
$dos = array('display', 'post','del');
$do = in_array($do, $dos) ? $do : 'display';
load()->model('mc');

if($do == 'display') {
	$_W['page']['title'] = '会员列表 - 会员 - 会员中心';
	global $_GPC, $_W;
	$groups = mc_groups();
	$pindex = max(1, intval($_GPC['page']));
	$psize = 50;
	$condition = '';
	$condition .= empty($_GPC['mobile']) ? '' : " AND `mobile` LIKE '%".trim($_GPC['mobile'])."%'";
	$condition .= empty($_GPC['email']) ? '' : " AND `email` LIKE '%".trim($_GPC['email'])."%'";
	$condition .= empty($_GPC['username']) ? '' : " AND `realname` LIKE '%".trim($_GPC['username'])."%'";
	$condition .= intval($_GPC['groupid']) > 0 ?  " AND `groupid` = '".intval($_GPC['groupid'])."'" : '';
	$list = pdo_fetchall("SELECT uid, uniacid, groupid, realname, nickname, email, mobile, createtime  FROM ".tablename('mc_members')." WHERE uniacid = '{$_W['uniacid']}' ".$condition." ORDER BY createtime DESC LIMIT " . ($pindex - 1) * $psize . ',' . $psize);
	$total = pdo_fetchcolumn("SELECT COUNT(*) FROM ".tablename('mc_members')." WHERE uniacid = '{$_W['uniacid']}' ".$condition);
	$pager = pagination($total, $pindex, $psize);
}

if($do == 'post') {
	$_W['page']['title'] = '编辑会员资料 - 会员 - 会员中心';
	$uid = intval($_GPC['uid']);
	if ($_W['ispost'] && $_W['isajax']) {
		$uid = $_GPC['uid'];
		$password = $_GPC['password'];
		$sql = 'SELECT `uid`, `salt` FROM ' . tablename('mc_members') . " WHERE `uniacid`=:uniacid AND `uid` = :uid";
		$user = pdo_fetch($sql, array(':uniacid' => $_W['uniacid'], ':uid' => $uid));
		if(empty($user) || $user['uid'] != $uid) {
			exit('error');
		}
		$password = md5($password . $user['salt'] . $_W['config']['setting']['authkey']);
		if (pdo_update('mc_members', array('password' => $password), array('uid' => $uid))) {
			exit('success');
		}
		exit('othererror');
	}
	if (checksubmit('submit')) {
		$uid = intval($_GPC['uid']);
		if (!empty($_GPC)) {
			if (!empty($_GPC['birth'])) {
				$_GPC['birthyear'] = $_GPC['birth']['year'];
				$_GPC['birthmonth'] = $_GPC['birth']['month'];
				$_GPC['birthday'] = $_GPC['birth']['day'];
			}
			if (!empty($_GPC['reside'])) {
				$_GPC['resideprovince'] = $_GPC['reside']['province'];
				$_GPC['residecity'] = $_GPC['reside']['city'];
				$_GPC['residedist'] = $_GPC['reside']['district'];
			}
			if (empty($_GPC['email']) && empty($_GPC['mobile'])) {
				$_GPC['email'] = md5($_GPC['openid']) . '@we7.cc';
			}
			unset($_GPC['uid']);
			$uid = mc_update($uid, $_GPC);
			if (!empty($_GPC['fanid']) && !empty($uid)) {
				pdo_update('mc_mapping_fans', array('uid' => $uid), array('fanid' => $_GPC['fanid']));
			}
		}
		message('更新资料成功！', referer(), 'success');
	}
	
	load()->func('tpl');
	$groups = mc_groups($_W['uniacid']);
	$profile = mc_fetch($uid);
	if(empty($uid)) {
		$fanid = intval($_GPC['fanid']);
		$tag = pdo_fetchcolumn('SELECT tag FROM ' . tablename('mc_mapping_fans') . ' WHERE uniacid = :uniacid AND fanid = :fanid', array(':uniacid' => $_W['uniacid'], ':fanid' => $fanid));
		$fan = iunserializer($tag) ? iunserializer($tag) : array();
		if(!empty($tag)) {
			if(!empty($fan['nickname'])) {
				$profile['nickname'] = $fan['nickname'];
			}
			if(!empty($fan['sex'])) {
				$profile['gender'] = $fan['sex'];
			}
			if(!empty($fan['city'])) {
				$profile['residecity'] = $fan['city'] . '市';
			}
			if(!empty($fan['province'])) {
				$profile['resideprovince'] = $fan['province'] . '省';
			}
			if(!empty($fan['country'])) {
				$profile['nationality'] = $fan['country'];
			}
			if(!empty($fan['headimgurl'])) {
				$profile['avatar'] = rtrim($fan['headimgurl'], '0') . 132;
			}
		}
	}
}

if($do == 'del') {
	$_W['page']['title'] = '删除会员资料 - 会员 - 会员中心';
	if(checksubmit('submit')) {
		if(!empty($_GPC['uid'])) {
			$instr = implode(',',$_GPC['uid']);
			pdo_query("DELETE FROM ".tablename('mc_members')." WHERE `uniacid` = {$_W['uniacid']} AND `uid` IN ({$instr})");
			message('删除成功！', referer(), 'success');
		}
		message('请选择要删除的项目！', referer(), 'error');
	}
}

template('mc/member');