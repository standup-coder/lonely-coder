<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
define('IN_SYS', true);
require '../framework/bootstrap.inc.php';
require IA_ROOT . '/web/common/bootstrap.sys.inc.php';
load()->web('common');
load()->web('template');

$acl = array(
	'account' => array(
		'default' => 'welcome',
		'direct' => array(
			'welcome'
		),
		'founder' => array(
			'batch',
			'permission',
			'groups'
		)
	),
	'home' => array(
		'default' => 'welcome',
		'founder' => array()
	),
	'cloud' => array(
		'default' => 'touch',
		'direct' => array(
			'touch',
			'dock',
			'download'
		),
		'founder' => array(
			'diagnose',
			'redirect',
			'upgrade',
			'process',
			'device'
		)
	),
	'extension' => array(
		'founder' => array(
			'module',
			'service',
			'theme',
		)
	),
	'site' => array(
		'direct' => array(
			'entry'
		)
	),
	'system' => array(
		'founder' => array(
			'common',
			'copyright',
			'database',
			'tools',
			'updatecache',
			'sysinfo'
		)
	),
	'user' => array(
		'default' => 'display',
		'direct' => array(
			'login',
			'register',
			'logout'
		),
		'founder' => array(
			'create',
			'display',
			'edit',
			'fields',
			'group',
			'permission',
			'registerset',
		)
	),
	'utility' => array(
		'founder' => array(
			'user'
		),
		'direct' => array(
			'verifycode',
			'code',
			'file',
			'emoji',
		)
	)
);

$controllers = array();
$handle = opendir(IA_ROOT . '/web/source/');
if(!empty($handle)) {
	while($dir = readdir($handle)) {
		if($dir != '.' && $dir != '..') {
			$controllers[] = $dir;
		}
	}
}
if(!in_array($controller, $controllers)) {
	$controller = 'account';
}
$init = IA_ROOT . "/web/source/{$controller}/__init.php";
if(is_file($init)) {
	require $init;
}

$actions = array();
$handle = opendir(IA_ROOT . '/web/source/' . $controller);
if(!empty($handle)) {
	while($dir = readdir($handle)) {
		if($dir != '.' && $dir != '..' && strexists($dir, '.ctrl.php')) {
			$dir = str_replace('.ctrl.php', '', $dir);
			$actions[] = $dir;
		}
	}
}
if(empty($actions)) {
	header('location: ?refresh');
}
if(!in_array($action, $actions)) {
	$action = $acl[$controller]['default'];
}
if(!in_array($action, $actions)) {
	$action = $actions[0];
}
$setting = setting_load('copyright');
$_W['page'] = array();
$_W['page']['copyright'] = $setting['copyright'];
unset($setting);

if(is_array($acl[$controller]['direct']) && in_array($action, $acl[$controller]['direct'])) {
		require _forward($controller, $action);
	exit;
}
if(is_array($acl[$controller]['founder']) && in_array($action, $acl[$controller]['founder'])) {
		if(!$_W['isfounder']) {
		message('不能访问, 需要创始人权限才能访问.');
	}
}
checklogin();
if(!defined('IN_GW')) {
	checkaccount();
	uni_group_check();
	if(!in_array($_W['role'], array('manager', 'operator', 'founder'))) {
		message('您的账号没有访问此公众号的权限.');
	}
}

$redirect = false;
if($_W['role'] == 'operator') {
	$limit = array();
	$limit['home'] = array(
		'welcome'
	);
	$limit['account'] = array(
		'display',
		'switch'
	);
	$limit['user'] = array(
		'profile'
	);
	$limit['system'] = array(
		'welcome'
	);
	$limit['utility'] = array(
		'emulator',
		'file'
	);
	if(!in_array($action, $limit[$controller])) {
		$redirect = true;
	}
	unset($limit);
}
if($redirect) {
	header('location: ' . url('home/welcome/solution'));
	exit;
} else {
	require _forward($controller, $action);
}
define('ENDTIME', microtime());
if (empty($_W['config']['setting']['maxtimeurl'])) {
	$_W['config']['setting']['maxtimeurl'] = 10;
}
if ((ENDTIME - STARTTIME) > $_W['config']['setting']['maxtimeurl']) {
	$data = array(
		'type' => '1',
		'runtime' => ENDTIME - STARTTIME,
		'runurl' => 'http://'.$_SERVER['HTTP_HOST'].$_SERVER['REQUEST_URI'],
		'createtime' => time()
	);
	pdo_insert('core_performance', $data);
}

function _forward($c, $a) {
	$file = IA_ROOT . '/web/source/' . $c . '/' . $a . '.ctrl.php';
	return $file;
}

function _calc_current_frames(&$frames) {
	global $controller, $action;
	if(!empty($frames) && is_array($frames)) {
		foreach($frames as &$frame) {
			foreach($frame['items'] as &$fr) {
				$query = parse_url($fr['url'], PHP_URL_QUERY);
				parse_str($query, $urls);
				if(defined('ACTIVE_FRAME_URL')) {
					$query = parse_url(ACTIVE_FRAME_URL, PHP_URL_QUERY);
					parse_str($query, $get);
				} else {
					$get = $_GET;
					$get['c'] = $controller;
					$get['a'] = $action;
				}
				if(!empty($do)) {
					$get['do'] = $do;
				}
				$diff = array_diff_assoc($urls, $get);
				if(empty($diff)) {
					$fr['active'] = ' active';
				}
			}
		}
	}
}