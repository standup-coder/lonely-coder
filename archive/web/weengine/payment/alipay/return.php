<?php
error_reporting(0);
define('IN_MOBILE', true);
if(empty($_GET['out_trade_no'])) {
	exit('request failed.');
}
$pieces = explode(':', $_GET['out_trade_no']);
if(!is_array($pieces) || count($pieces) != 2) {
	exit('request failed.');
}
require '../../framework/bootstrap.inc.php';
load()->app('common');
load()->app('template');
$_W['uniacid'] = $_W['weid'] = $pieces[0];
$setting = uni_setting($_W['uniacid'], array('payment'));
if(!is_array($setting['payment'])) {
	exit('request failed.');
}
$alipay = $setting['payment']['alipay'];
if(empty($alipay)) {
	exit('request failed.');
}
$prepares = array();
foreach($_GET as $key => $value) {
	if($key != 'sign' && $key != 'sign_type') {
		$prepares[] = "{$key}={$value}";
	}
}
sort($prepares);
$string = implode($prepares, '&');
$string .= $alipay['secret'];
$sign = md5($string);
if($sign == $_GET['sign'] && $_GET['result'] == 'success') {
	$plid = $pieces[1];
	$sql = 'SELECT * FROM ' . tablename('core_paylog') . ' WHERE `plid`=:plid';
	$params = array();
	$params[':plid'] = $plid;
	$log = pdo_fetch($sql, $params);
	if(!empty($log)) {

		$site = WeUtility::createModuleSite($log['module']);
		if(!is_error($site)) {
			$method = 'payResult';
			if (method_exists($site, $method)) {
				$ret = array();
				$ret['weid'] = $log['weid'];
				$ret['uniacid'] = $log['uniacid'];
				$ret['result'] = $log['status'] == '1' ? 'success' : 'failed';
				$ret['type'] = $log['type'];
				$ret['from'] = 'return';
				$ret['tid'] = $log['tid'];
				$ret['user'] = $log['openid'];
				$ret['fee'] = $log['fee'];
				exit($site->$method($ret));
			}
		}
	}
}
