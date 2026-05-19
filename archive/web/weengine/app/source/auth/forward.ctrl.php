<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');

$_W['setting']['authmode'] = empty($_W['setting']['authmode']) ? 1 : $_W['setting']['authmode'];
if($_GPC['__auth']) {
	$pass = @json_decode(base64_decode($_GPC['__auth']), true);
	if(is_array($pass) && !empty($pass['openid']) && !empty($pass['time']) && !empty($pass['hash'])) {
		if(($_W['setting']['authmode'] == 2 && abs($pass['time'] - TIMESTAMP) < 180) || $_W['setting']['authmode'] == 1) {
			$sql = 'SELECT `fanid`,`salt`,`uid` FROM ' . tablename('mc_mapping_fans') . ' WHERE `uniacid`=:uniacid AND `acid`=:acid AND `openid`=:openid';
			$pars = array();
			$pars[':uniacid'] = $_W['uniacid'];
			$pars[':acid'] = $pass['acid'];
			$pars[':openid'] = $pass['openid'];
			$fan = pdo_fetch($sql, $pars);
			if(!empty($fan)) {
				$hash = md5("{$pass['openid']}{$pass['time']}{$fan['salt']}{$_W['config']['setting']['authkey']}");
				if($pass['hash'] == $hash) {
					if ($_W['setting']['authmode'] == 2) {
						$rec = array();
						$rec['salt'] = random(8);
						pdo_update('mc_mapping_fans', $rec, array('uniacid' => $_W['acid'], 'openid' => $pass['openid']));
					}
					$_SESSION['acid'] = $pass['acid'];
					$_SESSION['openid'] = $pass['openid'];
				}
			}
		}
	}
}

$forward = @base64_decode($_GPC['forward']);
if(empty($forward)) {
	$forward = url('mc');
} else {
	$forward = (strexists($forward, 'http://') || strexists($forward, 'https://')) ? $forward : $_W['siteroot'] . 'app/' . $forward;
}
if(strexists($forward, '#')) {
	$pieces = explode('#', $forward, 2);
	$forward = $pieces[0];
}
$forward .= '&wxref=mp.weixin.qq.com#wechat_redirect';
header('location:' . $forward);


