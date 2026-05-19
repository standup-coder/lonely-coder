<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
load()->model('app');
$dos = array('display', 'credits', 'address', 'card', 'mycard', 'mobile');
$do = in_array($do, $dos) ? $do : 'display';
load()->func('tpl');
load()->model('user');


if ($do == 'credits') {
	$where = '';
	$params = array(':uid' => $_W['member']['uid']);
	$pindex = max(1, intval($_GPC['page']));
	$psize  = 15;
	
	if (empty($starttime) || empty($endtime)) {
		$starttime =  strtotime('-1 month');
		$endtime = time();
	}
	if ($_GPC['time']) {
		$starttime = strtotime($_GPC['time']['start']);
		$endtime = strtotime($_GPC['time']['end']) + 86399;
		$where = ' AND `createtime` >= :starttime AND `createtime` < :endtime';
		$params[':starttime'] = $starttime;
		$params[':endtime'] = $endtime;
	}
	if ($_GPC['credittype']) {
		
		if ($_GPC['type'] == 'order') {
			$sql = 'SELECT * FROM ' . tablename('mc_credits_recharge') . " WHERE `uid` = :uid $where LIMIT " . ($pindex - 1) * $psize. ',' . $psize;
			$orders = pdo_fetchall($sql, $params);
			foreach ($orders as &$value) {
				$value['createtime'] = date('Y-m-d', $value['createtime']);
				$value['fee'] = number_format($value['fee']);
				if ($value['status'] == 1) {
					$orderspay += $value['fee'];
				}
				unset($value);
			}
			
			$ordersql = 'SELECT COUNT(*) FROM ' .tablename('mc_credits_recharge') . "WHERE `uid` = :uid {$where}";
			$total = pdo_fetchcolumn($ordersql, $params);
			$orderpager = pagination($total, $pindex, $psize, '', array('before' => 0, 'after' => 0, 'ajaxcallback' => ''));
			template('mc/bond');
			exit();
		}
		$where .= " AND `credittype` = '{$_GPC['credittype']}'";
	}
	
	
	$sql = 'SELECT `num` FROM ' . tablename('mc_credits_record') . " WHERE `uid` = :uid $where";
	$nums = pdo_fetchall($sql, $params);
	$pay = $income = 0;
	foreach ($nums as $value) {
		if ($value['num'] > 0) {
			$income += $value['num'];
		} else {
			$pay += abs($value['num']);
		}
	}
	$pay = number_format($pay);
	$income = number_format($income);
	
	$sql = 'SELECT * FROM ' . tablename('mc_credits_record') . " WHERE `uid` = :uid {$where} ORDER BY `createtime` DESC LIMIT " . ($pindex - 1) * $psize.','. $psize;
	$data = pdo_fetchall($sql, $params);
	foreach ($data as $key=>$value) {
		$data[$key]['credittype'] = $creditnames[$data[$key]['credittype']]['title'];
		$data[$key]['createtime'] = date('Y-m-d', $data[$key]['createtime']);
		$data[$key]['num'] = number_format($value['num']);
	}
	
	$sql = 'SELECT `realname`, `avatar` FROM ' . tablename('mc_members') . " WHERE `uid` = :uid";
	$user = pdo_fetch($sql, array(':uid' => $_W['member']['uid']));
	
	$pagesql = 'SELECT COUNT(*) FROM ' .tablename('mc_credits_record') . "WHERE `uid` = :uid {$where}";
	$total = pdo_fetchcolumn($pagesql, $params);
	$pager = pagination($total, $pindex, $psize, '', array('before' => 0, 'after' => 0, 'ajaxcallback' => ''));
}


if ($do == 'address') {
	if (checksubmit('submit')) {
		
		$data = $_GPC['data'];
		$data['resideprovince'] = $_GPC['dis']['province'];
		$data['residecity'] = $_GPC['dis']['city'];
		$data['residedist'] = $_GPC['dis']['district'];
		pdo_update('mc_members', $data, array('uid'=>$_SESSION['uid']));
		message('修改收货地址成功', url('mc/bond/address'), 'success');
	}
	$sql = 'SELECT * FROM ' . tablename('mc_members') . " WHERE `uid` = :uid";
	$data = pdo_fetch($sql, array(':uid' => $_W['member']['uid']));
	$reside['province'] = $data['resideprovince'];
	$reside['city'] = $data['residecity'];
	$reside['district'] = $data['residedist'];
}


if ($do == 'card') {
	$mcard = pdo_fetch('SELECT * FROM ' . tablename('mc_card_members') . ' WHERE uniacid = :uniacid AND uid = :uid', array(':uniacid' => $_W['uniacid'], ':uid' => $_W['member']['uid']));
	if(!empty($mcard)) {
		header('Location:' . url('mc/bond/mycard'));
	}
	
	$sql = 'SELECT * FROM ' . tablename('mc_card') . "WHERE `uniacid` = :uniacid AND `status` = '1'";
	$setting = pdo_fetch($sql, array(':uniacid' => $_W['uniacid']));

	if (!empty($setting)) {
		$setting['color'] = iunserializer($setting['color']);
		$setting['background'] = iunserializer($setting['background']);
		$setting['fields'] = iunserializer($setting['fields']);
	} else {
		message('公众号尚未开启会员卡功能', url('mc'), 'error');
	}
	if(!empty($setting['fields'])) {
		$fields = array();
		foreach($setting['fields'] as $li) {
			if($li['bind'] == 'birth') {
				$fields[] = 'birthyear';
				$fields[] = 'birthmonth';
				$fields[] = 'birthday';
			} elseif($li['bind'] == 'reside') {
				$fields[] = 'resideprovince';
				$fields[] = 'residecity';
				$fields[] = 'residedist';
			} else {
				$fields[] = $li['bind'];
			}
		}
		$member_info = mc_fetch($_W['member']['uid'], $fields);
	}
	if (checksubmit('submit')) {
		$data = array();
		if (!empty($setting['fields'])) {
			foreach ($setting['fields'] as $row) {
				if (!empty($row['require']) && empty($_GPC[$row['bind']])) {
					message('请输入'.$row['title'].'！');
				}
				$data[$row['bind']] = $_GPC[$row['bind']];
			}
		}
		
		$sql = 'SELECT COUNT(*)  FROM ' . tablename('mc_card_members') . " WHERE `uid` = :uid AND `cid` = :cid AND uniacid = :uniacid";
		$count = pdo_fetchcolumn($sql, array(':uid' => $_W['member']['uid'], ':cid' => $_GPC['cardid'], ':uniacid' => $_W['uniacid']));
		if ($count >= 1) {
			message('抱歉,您已经领取过该会员卡.', referer(), 'error');
		}
		
 		$cardsn = $_GPC['format'];
		preg_match_all('/(\*+)/', $_GPC['format'], $matchs);
		if (!empty($matchs)) {
			foreach ($matchs[1] as $row) {
				$cardsn = str_replace($row, random(strlen($row), 1), $cardsn);
			}
		}
		preg_match('/(\#+)/', $_GPC['format'], $matchs);
		$length = strlen($matchs[1]);
		$pos = strpos($_GPC['format'], '#');
		$cardsn = str_replace($matchs[1], str_pad($_GPC['snpos']++, $length - strlen($number), '0', STR_PAD_LEFT), $cardsn);
		pdo_update('mc_card', array('snpos' => $_GPC['snpos']), array('uniacid' => $_W['uniacid'], 'id' => $_GPC['cardid']));
		
		$record = array(
				'uniacid' => $_W['uniacid'],
				'uid' => $_W['member']['uid'],
				'cid' => $_GPC['cardid'],
				'cardsn' => $cardsn,
				'status' => '1',
				'createtime' => TIMESTAMP
		);
		$check = mc_check($data);
		if(is_error($check)) {
			message($check['message'], '', 'error');
		}
		if(pdo_insert('mc_card_members', $record)) {
			if(!empty($data)){
				mc_update($_W['member']['uid'], $data);
			}
			message('领取会员卡成功.', url('mc/bond/mycard'), 'success');
		} else {
			message('领取会员卡失败.', referer(), 'error');
		}
	}
}


if ($do == 'mycard') {
	$mcard = pdo_fetch('SELECT * FROM ' . tablename('mc_card_members') . ' WHERE uniacid = :uniacid AND uid = :uid', array(':uniacid' => $_W['uniacid'], ':uid' => $_W['member']['uid']));
	if(empty($mcard)) {
		header('Location:' . url('mc/bond/card'));
	}
	$setting = pdo_fetch('SELECT * FROM ' . tablename('mc_card') . ' WHERE uniacid = :uniacid', array(':uniacid' => $_W['uniacid']));
	if(!empty($setting)) {
		$setting['color'] = iunserializer($setting['color']);
		$setting['background'] = iunserializer($setting['background']);
		$setting['business'] = iunserializer($setting['business']) ? iunserializer($setting['business']) : array();
	}
}

if($do == 'mobile') {
	$profile = mc_fetch($_W['member']['uid'], array('mobile'));
	$mobile_exist = empty($profile['mobile']) ? 0 : 1;
	if(checksubmit('submit')) {
		if($mobile_exist == 1) {
			$oldmobile = trim($_GPC['oldmobile']) ? trim($_GPC['oldmobile']) : message('请填写原手机号');
			$password = trim($_GPC['password']) ? trim($_GPC['password']) : message('请填写密码');
			$mobile = trim($_GPC['mobile']) ? trim($_GPC['mobile']) : message('请填写新手机号');
			if(!preg_match('/^\d{11}$/', $mobile)) {
				message('新手机号格式有误', '', 'error');
			}
			$info = pdo_fetch('SELECT uid, password, salt FROM ' . tablename('mc_members') . ' WHERE uniacid = :uniacid AND mobile = :mobile AND uid = :uid', array(':uniacid' => $_W['uniacid'], ':mobile' => $oldmobile, ':uid' => $_W['member']['uid']));
			if(!empty($info)) {
				if($info['password'] == md5($password . $info['salt'] . $_W['config']['setting']['authkey'])) {
					pdo_update('mc_members', array('mobile' => $mobile), array('uniacid' => $_W['uniacid'], 'uid' => $_W['member']['uid']));
					message('修改手机号成功', url('mc/home'), 'success');
				} else {
					message('密码输入错误', '', 'error');
				}
			} else {
				message('原手机号输入错误', '', 'error');
			}
		} else {
			$mobile = trim($_GPC['mobile']) ? trim($_GPC['mobile']) : message('请填写手机号');
			if(!preg_match('/^\d{11}$/', $mobile)) {
				message('手机号格式有误', '', 'error');
			}
			$is_exist = pdo_fetch('SELECT uid FROM ' . tablename('mc_members') . ' WHERE uniacid = :uniacid AND mobile = :mobile AND uid != :uid', array(':uniacid' => $_W['uniacid'], ':mobile' => $mobile, ':uid' => $_W['member']['uid']));
			if(!empty($is_exist)) {
				message('该手机号已被绑定,换个手机号试试', '', 'error');
			}
			pdo_update('mc_members', array('mobile' => $mobile), array('uniacid' => $_W['uniacid'], 'uid' => $_W['member']['uid']));
			message('修改手机号成功', url('mc/home'), 'success');
		}
	}
}
template('mc/bond');