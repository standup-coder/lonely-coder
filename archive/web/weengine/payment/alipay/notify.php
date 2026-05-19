<?php
error_reporting(0);
define('IN_MOBILE', true);

$obj = simplexml_load_string($_POST['notify_data'], 'SimpleXMLElement', LIBXML_NOCDATA);
if($obj instanceof SimpleXMLElement && $obj->out_trade_no) {
	$out_trade_no = strval($obj->out_trade_no);
	$pieces = explode(':', $out_trade_no);
	if(is_array($pieces) && count($pieces) == 2) {
		require '../../framework/bootstrap.inc.php';
		$_W['uniacid'] = $_W['weid'] = $pieces[0];
		$setting = uni_setting($_W['uniacid'], array('payment'));
		if(is_array($setting['payment'])) {
			$alipay = $setting['payment']['alipay'];
			if(!empty($alipay)) {
				$string = "service={$_POST['service']}&v={$_POST['v']}&sec_id={$_POST['sec_id']}&notify_data={$_POST['notify_data']}";
				$string .= $alipay['secret'];
				$sign = md5($string);
				if($sign == $_POST['sign']) {
					$plid = $pieces[1];
					$sql = 'SELECT * FROM ' . tablename('core_paylog') . ' WHERE `plid`=:plid';
					$params = array();
					$params[':plid'] = $plid;
					$log = pdo_fetch($sql, $params);
					if(!empty($log) && $log['status'] == '0') {
						$record = array();
						$record['status'] = '1';
						pdo_update('core_paylog', $record, array('plid' => $log['plid']));
						
						$site = WeUtility::createModuleSite($log['module']);
						if(!is_error($site)) {
							$method = 'payResult';
							if (method_exists($site, $method)) {
								$ret = array();
								$ret['weid'] = $log['weid'];
								$ret['uniacid'] = $log['uniacid'];
								$ret['result'] = 'success';
								$ret['type'] = $log['type'];
								$ret['from'] = 'notify';
								$ret['tid'] = $log['tid'];
								$ret['user'] = $log['openid'];
								$ret['fee'] = $log['fee'];
								$site->$method($ret);
								exit('success');
							}
						}
					}
				}
			}
		}
	}
}
exit('fail');
