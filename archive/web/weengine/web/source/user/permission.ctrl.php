<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
$_W['page']['title'] = '查看用户权限 - 用户管理 - 用户管理';
load()->model('setting');
$uid = intval($_GPC['uid']);
$m = array();
$m['uid'] = $uid;
$member = user_single($m);
$founders = explode(',', $_W['config']['setting']['founder']);
if(empty($member) || in_array($m['uid'], $founders)) {
	message('访问错误.');
}

$do = $_GPC['do'];
$dos = array('deny', 'delete', 'auth', 'revo', 'revos', 'select', 'role');
$do = in_array($do, $dos) ? $do: 'edit';
if($do == 'edit') {
		if (!empty($member['groupid'])) {
		$group = pdo_fetch("SELECT * FROM ".tablename('users_group')." WHERE id = '{$member['groupid']}'");
		if (!empty($group)) {
			$package = iunserializer($group['package']);
			$group['package'] = uni_groups($package);
		}
	}
	$weids = pdo_fetchall("SELECT uniacid, role FROM ".tablename('uni_account_users')." WHERE uid = '$uid'", array(), 'uniacid');
	if (!empty($weids)) {
		$wechats = pdo_fetchall("SELECT * FROM ".tablename('uni_account')." WHERE uniacid IN (".implode(',', array_keys($weids)).")");
	}
	template('user/permission');
}

if($do == 'deny') {
	if($_W['ispost'] && $_W['isajax']) {
		$founders = explode(',', $_W['config']['setting']['founder']);
		if(in_array($uid, $founders)) {
			exit('管理员用户不能禁用.');
		}
		$member = array();
		$member['uid'] = $uid;
		$status = $_GPC['status'];
		$member['status'] = $status == '-1' ? '-1' : '0';
		if(user_update($member)) {
			exit('success');
		}
	}
}

if ($do == 'select') {
	$uid = intval($_GPC['uid']);
	$condition = '';
	$params = array();
	if(!empty($_GPC['keyword'])) {
		$condition = ' AND `name` LIKE :name';
		$params[':name'] = "%{$_GPC['keyword']}%";
	}
	$pindex = max(1, intval($_GPC['page']));
	$psize = 10;
	$total = 0;
	
	$list = pdo_fetchall("SELECT * FROM ".tablename('uni_account')." WHERE 1 $condition LIMIT ".(($pindex - 1) * $psize).",{$psize}");
	$total = pdo_fetchcolumn("SELECT COUNT(*) FROM ".tablename('uni_account')." WHERE 1 $condition");
	$pager = pagination($total, $pindex, $psize, '', array('ajaxcallback'=>'null'));
	
	$permission = pdo_fetchall("SELECT uniacid FROM ".tablename('uni_account_users')." WHERE uid = '$uid'", array(), 'uniacid');
	template('user/select');
}

if ($do == 'auth') {
	$uniacid = intval($_GPC['uniacid']);
	$uid = intval($uid);
	
	$isexists = pdo_fetch("SELECT * FROM ".tablename('uni_account_users')." WHERE uid = :uid AND uniacid = :uniacid", array(':uid' => $uid, ':uniacid' => $uniacid));
	if (empty($isexists)) {
		pdo_insert('uni_account_users', array('uniacid' => $uniacid, 'uid' => $uid));
	}
	exit('success');
}

if ($do == 'revo') {
	$uniacid = intval($_GPC['uniacid']);
	$uid = intval($uid);
	
	$isexists = pdo_fetch("SELECT * FROM ".tablename('uni_account_users')." WHERE uid = :uid AND uniacid = :uniacid", array(':uid' => $uid, ':uniacid' => $uniacid));
	if (!empty($isexists)) {
		pdo_delete('uni_account_users', array('uniacid' => $uniacid, 'uid' => $uid));
	}
	exit('success');
}

if ($do == 'role') {
	$uid = intval($_GPC['uid']);
	$uniacid = intval($_GPC['uniacid']);
	$role = !empty($_GPC['role']) && in_array($_GPC['role'], array('operator', 'manager')) ? $_GPC['role'] : 'operator';
	pdo_update('uni_account_users', array('role' => $role), array('uid' => $uid, 'uniacid' => $uniacid));
}
