<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');


function url($segment, $params = array(), $noredirect = false) {
	return murl($segment, $params, $noredirect);
}


function message($msg, $redirect = '', $type = '') {
	global $_W;
	if($redirect == 'refresh') {
		$redirect = $_W['script_name'] . '?' . $_SERVER['QUERY_STRING'];
	}
	if($redirect == 'referer') {
		$redirect = referer();
	}
	if($redirect == '') {
		$type = in_array($type, array('success', 'error', 'info', 'warning', 'ajax', 'sql')) ? $type : 'info';
	} else {
		$type = in_array($type, array('success', 'error', 'info', 'warning', 'ajax', 'sql')) ? $type : 'success';
	}
	if($_W['isajax'] || $type == 'ajax') {
		$vars = array();
		$vars['message'] = $msg;
		$vars['redirect'] = $redirect;
		$vars['type'] = $type;
		exit(json_encode($vars));
	}
	if (empty($msg) && !empty($redirect)) {
		header('location: '.$redirect);
	}
	$label = $type;
	if($type == 'error') {
		$label = 'danger';
	}
	if($type == 'ajax' || $type == 'sql') {
		$label = 'warning';
	}
	include template('common/message', TEMPLATE_INCLUDEPATH);
	exit();
}


function checkauth() {
	global $_W;
	load()->model('mc');
	if(!empty($_W['member']) && (!empty($_W['member']['mobile']) || !empty($_W['member']['email']))) {
		return true;
	}
	if(!empty($_W['openid'])) {
		$sql = 'SELECT `fanid`,`openid`,`uid` FROM ' . tablename('mc_mapping_fans') . ' WHERE `uniacid`=:uniacid AND `openid`=:openid';
		$pars = array();
		$pars[':uniacid'] = $_W['uniacid'];
		$pars[':openid'] = $_W['openid'];
		$fan = pdo_fetch($sql, $pars);
		if(!empty($fan) && !empty($fan['uid'])) {
			if(_mc_login(array('uid' => $fan['uid']))) {
				return true;
			} else {
				$rec = array();
				$rec['uid'] = $fan['uid'] = 0;
				pdo_update('mc_mapping_fans', $rec, array('fanid' => $fan['fanid']));
			}
		}
		$setting = uni_setting($_W['uniacid'], array('passport'));
		if (empty($setting['passport']['focusreg'])) {
						$default_groupid = pdo_fetchcolumn('SELECT groupid FROM ' .tablename('mc_groups') . ' WHERE uniacid = :uniacid AND isdefault = 1', array(':uniacid' => $_W['uniacid']));
			$data = array(
				'uniacid' => $_W['uniacid'],
				'email' => md5($_W['openid']).'@we7.cc',
				'salt' => random(8),
				'groupid' => $default_groupid, 
				'createtime' => TIMESTAMP,
			);
			$data['password'] = md5($_W['openid'] . $data['salt'] . $_W['config']['setting']['authkey']);
			pdo_insert('mc_members', $data);
			$user['uid'] = pdo_insertid();
			pdo_update('mc_mapping_fans', array('uid' => $user['uid']), array('fanid' => $fan['fanid']));
			if(_mc_login($user)) {
				return true;
			}
		}
	}
	$forward = base64_encode($_SERVER['QUERY_STRING']);
	if($_W['isajax']) {
		$result = array();
		$result['url'] = url('auth/login', array('forward' => $forward), true);
		$result['act'] = 'redirect';
		exit(json_encode($result));
	} else {
		header("location: " . url('auth/login', array('forward' => $forward)), true);
	}
	exit;
}


function init_quickmenus($multiid = 0) {
	global $_W, $controller, $action;
	$quickmenu = pdo_fetchcolumn('SELECT quickmenu FROM ' . tablename('site_multi') . ' WHERE uniacid = :uniacid AND id = :id', array(':uniacid' => $_W['uniacid'], ':id' => $multiid));
	$quickmenu = iunserializer($quickmenu);
	if(!empty($quickmenu)) {
		$acl = array();
		$acl['home'] = array(
			'home'
		);
		if((is_array($acl[$controller]) && in_array($action, $acl[$controller])) || in_array(IN_MODULE, $quickmenu['enablemodule'])) {
			$tpl = !empty($quickmenu['template']) ? $quickmenu['template'] : 'default';
			$_W['quickmenu']['menus'] = app_navs('shortcut', $multiid);
			$_W['quickmenu']['template'] = '../quick/' . $tpl;
		}
	}
}
