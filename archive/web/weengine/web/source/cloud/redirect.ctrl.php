<?php 
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */

if(empty($_W['isfounder'])) {
	message('访问非法.');
}
$do = in_array($do, array('profile', 'device', 'callback', 'appstore')) ? $do : 'profile';
$authurl = 'http://v2.addons.we7.cc/web/index.php?c=auth&a=passwort';

$auth = array();
$auth['key'] = '';
$auth['password'] = '';
$auth['url'] = rtrim($_W['siteroot'], '/');
$auth['referrer'] = intval($_W['config']['setting']['referrer']);
$auth['version'] = IMS_VERSION;
if(!empty($_W['setting']['site']['key']) && !empty($_W['setting']['site']['token'])) {
	$auth['key'] = $_W['setting']['site']['key'];
	$auth['password'] = md5($_W['setting']['site']['key'] . $_W['setting']['site']['token']);
}

if($do == 'profile') {
	$auth['forward'] = 'profile';
	$iframe = __to($auth);
	$title = '注册站点';
}

if($do == 'appstore') {
	$auth['forward'] = 'appstore';
	$iframe = __to($auth);
	$title = '应用商城';
}

if($do == 'device') {
	$auth['forward'] = 'device';
	$iframe = __to($auth);
	$title = '微擎设备';
}

if($do == 'promotion') {
	if(empty($_W['setting']['site']['key']) || empty($_W['setting']['site']['token'])) {
		message("你的程序需要在微擎云服务平台注册你的站点资料, 来接入云平台服务后才能使用推广功能.", url('cloud/profile'), 'error');
	}
	$auth['forward'] = 'promotion';
	$iframe = __to($auth);
	$title = '我要推广';
}

if($do == 'callback') {
	$secret = $_GPC['token'];
	if(strlen($secret) == 32) {
		$cache = cache_read('cloud:auth:transfer');
		cache_delete('cloud:auth:transfer');
		if(!empty($cache) && $cache['secret'] == $secret) {
			$site = array_elements(array('key', 'token'), $cache);
			setting_save($site, 'site');
			$auth['key'] = $site['key'];
			$auth['password'] = md5($site['key'] . $site['token']);
			$auth['forward'] = 'profile';
			header('location: ' . __to($auth));
			exit();
		}
	}
	message('访问错误.');
}

template('cloud/frame');

function __to($auth) {
	global $authurl;
	$query = base64_encode(json_encode($auth));
	return $authurl . '&__auth=' . $query;
}
