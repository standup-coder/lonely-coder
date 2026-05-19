<?php 
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
$moduels = uni_modules();

$params = @json_decode(base64_decode($_GPC['params']), true);
if(empty($params) || !array_key_exists($params['module'], $moduels)) {
	message('访问错误.');
}
$setting = uni_setting($_W['uniacid'], 'payment');
$dos = array();
if(!empty($setting['payment']['credit']['switch'])) {
	$dos[] = 'credit';
}
if(!empty($setting['payment']['alipay']['switch'])) {
	$dos[] = 'alipay';
}
if(!empty($setting['payment']['wechat']['switch'])) {
	$dos[] = 'wechat';
}
if(!empty($setting['payment']['delivery']['switch'])) {
	$dos[] = 'delivery';
}

$do = $_GET['do'];
$type = in_array($do, $dos) ? $do : '';
if(empty($type)) {
	message('支付方式错误,请联系商家', '', 'error');
}

if(!empty($type)) {
	$sql = 'SELECT * FROM ' . tablename('core_paylog') . ' WHERE `uniacid`=:uniacid AND `module`=:module AND `tid`=:tid';
	$pars  = array();
	$pars[':uniacid'] = $_W['uniacid'];
	$pars[':module'] = $params['module'];
	$pars[':tid'] = $params['tid'];
	$log = pdo_fetch($sql, $pars);
	if(!empty($log) && $log['status'] != '0') {
		message('这个订单已经支付成功, 不需要重复支付.');
	}
	if($log['fee'] != $params['fee']) {
		pdo_delete('core_paylog', array('plid' => $log['plid']));
		$log = null;
	}
	if(empty($log)) {
		$fee = $params['fee'];
		$record = array();
		$record['uniacid'] = $_W['uniacid'];
		$record['openid'] = $_W['member']['uid'];
		$record['module'] = $params['module'];
		$record['type'] = $type;
		$record['tid'] = $params['tid'];
		$record['fee'] = $fee;
		$record['status'] = '0';
		if(pdo_insert('core_paylog', $record)) {
			$plid = pdo_insertid();
			$record['plid'] = $plid;
			$log = $record;
		} else {
			message('系统错误, 请稍后重试.');
		}
	} else {
		if($log['type'] != $type) {
			$record = array();
			$record['type'] = $type;
			pdo_update('core_paylog', $record, array('plid' => $log['plid']));
		}
	}
	$ps = array();
	$ps['tid'] = $log['plid'];
	$ps['user'] = $_W['fans']['from_user'];
	$ps['fee'] = $log['fee'];
	$ps['title'] = $params['title'];
	if($type == 'alipay') {
		load()->model('payment');
		load()->func('communication');
		$ret = alipay_build($ps, $setting['payment']['alipay']);
		if($ret['url']) {
			header("location: {$ret['url']}");
			exit();
		}
	}
	if($type == 'wechat') {
		load()->model('payment');
		load()->func('communication');
		$sl = base64_encode(json_encode($ps));
		$auth = sha1($sl . $_W['uniacid'] . $_W['config']['setting']['authkey']);
		header("location: ./payment/wechat/pay.php?weid={$_W['uniacid']}&auth={$auth}&ps={$sl}");
		exit();
	}
	if($type == 'credit') {
		$setting = uni_setting($_W['uniacid'], array('creditbehaviors'));
		$credtis = mc_credit_fetch($_W['member']['uid']);
		
		if($credtis[$setting['creditbehaviors']['currency']] < $ps['fee']) {
			message("余额不足以支付, 需要 {$ps['fee']}, 当前 {$credtis[$setting['creditbehaviors']['currency']]}");
		}
		$fee = floatval($ps['fee']);
		$result = mc_credit_update($_W['member']['uid'], $setting['creditbehaviors']['currency'], -$fee, array($_W['member']['uid'], '消费' . $setting['creditbehaviors']['currency'] . ':' . $fee));
		if (is_error($result)) {
			message($result['message'], '', 'error');
		}
		$sql = 'SELECT * FROM ' . tablename('core_paylog') . ' WHERE `plid`=:plid';
		$pars = array();
		$pars[':plid'] = $ps['tid'];
		$log = pdo_fetch($sql, $pars);
		if(!empty($log) && $log['status'] == '0') {
			$record = array();
			$record['status'] = '1';
			pdo_update('core_paylog', $record, array('plid' => $log['plid']));

			$site = WeUtility::createModuleSite($log['module']);
			if(!is_error($site)) {
				$site->module = $_W['account']['modules'][$log['module']];
				$site->weid = $_W['weid'];
				$site->uniacid = $_W['uniacid'];
				$site->inMobile = true;
				$method = 'payResult';
				if (method_exists($site, $method)) {
					$ret = array();
					$ret['result'] = 'success';
					$ret['type'] = $log['type'];
					$ret['from'] = 'return';
					$ret['tid'] = $log['tid'];
					$ret['user'] = $log['openid'];
					$ret['fee'] = $log['fee'];
					$ret['weid'] = $log['weid'];
					$ret['uniacid'] = $log['uniacid'];
					exit($site->$method($ret));
				}
			}
		}
	}
	
	if ($type == 'delivery') {
		$sql = 'SELECT * FROM ' . tablename('core_paylog') . ' WHERE `plid`=:plid';
		$pars = array();
		$pars[':plid'] = $ps['tid'];
		$log = pdo_fetch($sql, $pars);
		if(!empty($log) && $log['status'] == '0') {
			$site = WeUtility::createModuleSite($log['module']);

			if(!is_error($site)) {
				$site->module = $_W['account']['modules'][$log['module']];
				$site->weid = $_W['weid'];
				$site->uniacid = $_W['uniacid'];
				$site->inMobile = true;
				$method = 'payResult';
				if (method_exists($site, $method)) {
					$ret = array();
					$ret['result'] = 'failed';
					$ret['type'] = $log['type'];
					$ret['from'] = 'return';
					$ret['tid'] = $log['tid'];
					$ret['user'] = $log['openid'];
					$ret['fee'] = $log['fee'];
					$ret['weid'] = $log['weid'];
					$ret['uniacid'] = $log['uniacid'];
					exit($site->$method($ret));
				}
			}
		}
	}
}
