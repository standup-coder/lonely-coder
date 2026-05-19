<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
load()->model('mc');

$uniacid = intval($_GPC['weid']);
if(empty($uniacid)) {
	$uniacid = intval($_GPC['i']); 
}
$_W['account'] = uni_fetch($uniacid);
if(empty($_W['account'])) {
	exit('error mobile site');
}
$_W['uniacid'] = $_W['account']['uniacid'];
$_W['acid'] = '0';

$ts = uni_templates();

$setting = uni_setting($_W['uniacid'], 'default_site');
$multiid = intval($setting['default_site']);
if(!empty($_GPC['t'])) {
	$multiid = intval($_GPC['t']);
}

$multi = pdo_fetch("SELECT * FROM ".tablename('site_multi')." WHERE id = :id", array(':id' => $multiid));
if(empty($multi)) {
	exit('没有找到指定微站');
}
$styleid = $multi['styleid'];
if(!empty($_GPC['s'])) {
	$styleid = intval($_GPC['s']);
}
$style = pdo_fetch("SELECT * FROM ".tablename('site_styles')." WHERE id = :id", array(':id' => $styleid));
$setting = array('styleid' => $style['templateid']);

$_W['template'] = !empty($ts[$setting['styleid']]) ? $ts[$setting['styleid']]['name'] : 'default';
if(!empty($style['id'])) {
	$sql = "SELECT * FROM " . tablename('site_styles_vars') . " WHERE `uniacid`=:uniacid AND `styleid`=:styleid";
	$pars = array();
	$pars[':uniacid'] = $_W['uniacid'];
	$pars[':styleid'] = $style['id'];
	$ds = pdo_fetchall($sql, $pars);
	if(!empty($ds)) {
		foreach($ds as $row) {
			if (strexists($row['variable'], 'img')) {
				$row['content'] = tomedia($row['content']);
			}
			$_W['styles'][$row['variable']] = $row['content'];
		}
	}
}

$_W['page'] = array();
$site_info = iunserializer($multi['site_info']);
if(is_array($site_info)) {
	$_W['page'] = array_merge($_W['page'], $site_info);
}

$_W['container'] = 'browser';
if(strexists(strtolower($_SERVER['HTTP_USER_AGENT']), 'micromessenger')) {
	$_W['container'] = 'wechat';
}
if(strexists(strtolower($_SERVER['HTTP_USER_AGENT']), 'yixin')) {
	$_W['container'] = 'yixin';
}
if(!strexists(strtolower($_SERVER['HTTP_USER_AGENT']), 'mobile')) {
	$_W['container'] = 'web';
}

$sessid = $_COOKIE[session_name()];
if(!empty($sessid)) {
	$pieces = explode('-', $sessid);
	if(count($pieces) != 2 || $pieces[0] != $_W['uniacid']) {
		$sessid = '';
	}
}
if(empty($sessid)) {
	$sessid = $_W['uniacid'] . '-' . random(20);
}
session_id($sessid);
session_start();

if(!empty($_SESSION['acid'])) {
	$sql = 'SELECT * FROM ' . tablename('account') . ' WHERE `uniacid`=:uniacid AND `acid`=:acid';
	$pars = array();
	$pars[':uniacid'] = $_W['uniacid'];
	$pars[':acid'] = $_SESSION['acid'];
	if(pdo_fetch($sql, $pars)) {
		$_W['acid'] = $_SESSION['acid'];
	}
}
if(!empty($_SESSION['openid'])) {
	$_W['openid'] = $_SESSION['openid'];
	if(!empty($_W['acid'])) {
		$sql = 'SELECT * FROM ' . tablename('mc_mapping_fans') . ' WHERE `uniacid`=:uniacid AND `acid`=:acid AND `openid`=:openid LIMIT 1';
		$pars = array();
		$pars[':uniacid'] = $_W['uniacid'];
		$pars[':acid'] = $_W['acid'];
		$pars[':openid'] = $_W['openid'];
		$mapping = pdo_fetch($sql, $pars);
		if(!empty($mapping)) {
			_mc_login(array('uid' => $mapping['uid']));
		}
	}
}
if(!empty($_SESSION['uid'])) {
	_mc_login(array('uid' => $_SESSION['uid']));
}
if(empty($_W['openid'])) {
	$setting = uni_setting($_W['uniacid'], array('oauth'));
	$oauth = $setting['oauth'];
	if(!empty($oauth['status']) && !empty($oauth['account'])) {
		$account = account_fetch($oauth['account']);
		if($account['type'] == '1' && $_W['container'] == 'wechat') { 			$callback = urlencode($_W['siteroot'] . 'app/index.php?c=auth&a=oauth&uniacid=' . $_W['uniacid']);
			$state = base64_encode($_SERVER['QUERY_STRING']);
			$stateKey = substr(md5($state),0,16);
			$_SESSION[$stateKey] = $state;
			$state = $stateKey;
			$forward = "https://open.weixin.qq.com/connect/oauth2/authorize?appid={$account['key']}&redirect_uri={$callback}&response_type=code&scope=snsapi_base&state={$state}#wechat_redirect";
			header('location: ' . $forward);
			exit();
		}
	}
}

load()->func('compat.biz');
