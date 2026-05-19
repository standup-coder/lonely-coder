<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');

define('ALIPAY_GATEWAY', 'http://wappaygw.alipay.com/service/rest.htm');


function alipay_build($params, $alipay = array()) {
	global $_W;
	$tid = $_W['uniacid'] . ':' . $params['tid'];
	$set = array();
	$set['service'] = 'alipay.wap.trade.create.direct';
	$set['format'] = 'xml';
	$set['v'] = '2.0';
	$set['partner'] = $alipay['partner'];
	$set['req_id'] = $tid;
	$set['sec_id'] = 'MD5';
	$callback = $_W['siteroot'] . 'payment/alipay/return.php';
	$notify = $_W['siteroot'] . 'payment/alipay/notify.php';
	$merchant = $_W['siteroot'] . 'payment/alipay/merchant.php';
	$expire = 10;
	$set['req_data'] = "<direct_trade_create_req><subject>{$params['title']}</subject><out_trade_no>{$tid}</out_trade_no><total_fee>{$params['fee']}</total_fee><seller_account_name>{$alipay['account']}</seller_account_name><call_back_url>{$callback}</call_back_url><notify_url>{$notify}</notify_url><out_user>{$params['user']}</out_user><merchant_url>{$merchant}</merchant_url><pay_expire>{$expire}</pay_expire></direct_trade_create_req>";
	$prepares = array();
	foreach($set as $key => $value) {
		if($key != 'sign') {
			$prepares[] = "{$key}={$value}";
		}
	}
	sort($prepares);
	$string = implode($prepares, '&');
	$string .= $alipay['secret'];
	$set['sign'] = md5($string);
	$response = ihttp_get(ALIPAY_GATEWAY . '?' . http_build_query($set));
	$ret = array();
	@parse_str($response['content'], $ret);
	foreach($ret as &$v) {
		$v = str_replace('\"', '"', $v);
	}
	if(is_array($ret)) {
		if($ret['res_error']) {
			$error = simplexml_load_string($ret['res_error'], 'SimpleXMLElement', LIBXML_NOCDATA);
			if($error instanceof SimpleXMLElement && $error->detail) {
				message("发生错误, 无法继续支付. 详细错误为: " . strval($error->detail));
			}
		}

		if($ret['partner'] == $set['partner'] && $ret['req_id'] == $set['req_id'] && $ret['sec_id'] == $set['sec_id'] && $ret['service'] == $set['service'] && $ret['v'] == $set['v']) {
			$prepares = array();
			foreach($ret as $key => $value) {
				if($key != 'sign') {
					$prepares[] = "{$key}={$value}";
				}
			}
			sort($prepares);
			$string = implode($prepares, '&');
			$string .= $alipay['secret'];
			if(md5($string) == $ret['sign']) {
				$obj = simplexml_load_string($ret['res_data'], 'SimpleXMLElement', LIBXML_NOCDATA);
				if($obj instanceof SimpleXMLElement && $obj->request_token) {
					$token = strval($obj->request_token);
					$set = array();
					$set['service'] = 'alipay.wap.auth.authAndExecute';
					$set['format'] = 'xml';
					$set['v'] = '2.0';
					$set['partner'] = $alipay['partner'];
					$set['sec_id'] = 'MD5';
					$set['req_data'] = "<auth_and_execute_req><request_token>{$token}</request_token></auth_and_execute_req>";
					$prepares = array();
					foreach($set as $key => $value) {
						if($key != 'sign') {
							$prepares[] = "{$key}={$value}";
						}
					}
					sort($prepares);
					$string = implode($prepares, '&');
					$string .= $alipay['secret'];
					$set['sign'] = md5($string);
					$url = ALIPAY_GATEWAY . '?' . http_build_query($set);
					return array('url' => $url);
				}
			}
		}
	}
	message('非法访问.');
}



function wechat_build($params, $wechat) {
	global $_W;
	if (empty($wechat['version']) && !empty($wechat['signkey'])) {
		$wechat['version'] = 1;
	}
	$wOpt = array();
	if ($wechat['version'] == 1) {
		$wOpt['appId'] = $_W['account']['key'];
		$wOpt['timeStamp'] = TIMESTAMP;
		$wOpt['nonceStr'] = random(8);
		$package = array();
		$package['bank_type'] = 'WX';
		$package['body'] = $params['title'];
		$package['attach'] = $_W['weid'];
		$package['partner'] = $wechat['partner'];
		$package['out_trade_no'] = $params['tid'];
		$package['total_fee'] = $params['fee'] * 100;
		$package['fee_type'] = '1';
		$package['notify_url'] = $_W['siteroot'] . 'notify.php'; 		$package['spbill_create_ip'] = CLIENT_IP;
		$package['time_start'] = date('YmdHis', TIMESTAMP);
		$package['time_expire'] = date('YmdHis', TIMESTAMP + 600);
		$package['input_charset'] = 'UTF-8';
		ksort($package);
		$string1 = '';
		foreach($package as $key => $v) {
			$string1 .= "{$key}={$v}&";
		}
		$string1 .= "key={$wechat['key']}";
		$sign = strtoupper(md5($string1));

		$string2 = '';
		foreach($package as $key => $v) {
			$v = urlencode($v);
			$string2 .= "{$key}={$v}&";
		}
		$string2 .= "sign={$sign}";
		$wOpt['package'] = $string2;

		$string = '';
		$keys = array('appId', 'timeStamp', 'nonceStr', 'package', 'appKey');
		sort($keys);
		foreach($keys as $key) {
			$v = $wOpt[$key];
			if($key == 'appKey') {
				$v = $wechat['signkey'];
			}
			$key = strtolower($key);
			$string .= "{$key}={$v}&";
		}
		$string = rtrim($string, '&');

		$wOpt['signType'] = 'SHA1';
		$wOpt['paySign'] = sha1($string);
		return $wOpt;
	} else {
		$package = array();
		$package['appid'] = $_W['account']['key'];
		$package['mch_id'] = $wechat['mchid'];
		$package['nonce_str'] = random(8);
		$package['body'] = $params['title'];
		$package['attach'] = $_W['weid'];
		$package['out_trade_no'] = $params['tid'];
		$package['total_fee'] = $params['fee'] * 100;
		$package['spbill_create_ip'] = CLIENT_IP;
		$package['time_start'] = date('YmdHis', TIMESTAMP);
		$package['time_expire'] = date('YmdHis', TIMESTAMP + 600);
		$package['notify_url'] = $_W['siteroot'] . 'notify.php'; 		$package['trade_type'] = 'JSAPI';
		$package['openid'] = $_W['openid'];

								
		ksort($package, SORT_STRING);
		$string1 = '';
		foreach($package as $key => $v) {
			$string1 .= "{$key}={$v}&";
		}
		$string1 .= "key={$wechat['key']}";
		$package['sign'] = strtoupper(md5($string1));
		$dat = array2xml($package);
		$response = ihttp_request('https://api.mch.weixin.qq.com/pay/unifiedorder', $dat);
		if (is_error($response)) {
			return $response;
		}
		$xml = @simplexml_load_string($response['content'], 'SimpleXMLElement', LIBXML_NOCDATA);
		if (strval($xml->return_code) == 'FAIL') {
			return error(-1, strval($xml->return_msg));
		}
		if (strval($xml->result_code) == 'FAIL') {
			return error(-1, strval($xml->err_code).': '.strval($xml->err_code_des));
		}
		$prepayid = $xml->prepay_id;
		$wOpt['appId'] = $_W['account']['key'];
		$wOpt['timeStamp'] = TIMESTAMP;
		$wOpt['nonceStr'] = random(8);
		$wOpt['package'] = 'prepay_id='.$prepayid;
		$wOpt['signType'] = 'MD5';
		ksort($wOpt, SORT_STRING);
		foreach($wOpt as $key => $v) {
			$string .= "{$key}={$v}&";
		}
		$string .= "key={$wechat['key']}";
		$wOpt['paySign'] = strtoupper(md5($string));
		return $wOpt;
	}
}
