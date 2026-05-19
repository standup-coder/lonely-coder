<?php 
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
$_W['page']['title'] = '用户列表 - 用户管理 - 用户管理';
$pindex = max(1, intval($_GPC['page']));
$psize = 20;

$where = '';
if (isset($_GPC['status']) && $_GPC['status'] !== '') {
	$where .= " AND status = '".intval($_GPC['status'])."'";
}
if (!empty($_GPC['username'])) {
	$where .= " AND username LIKE '%{$_GPC['username']}%'";
}

if (!empty($_GPC['group'])) {
	$where .= " AND groupid = '{$_GPC['group']}'";
}
$sql = 'SELECT * FROM ' . tablename('users') . " WHERE 1 " .$where . " LIMIT " . ($pindex - 1) * $psize .',' .$psize;
$members = pdo_fetchall($sql);
$total = pdo_fetchcolumn('SELECT COUNT(*) FROM ' . tablename('users') . ' WHERE 1 ' . $where);
$pager = pagination($total, $pindex, $psize);

$founders = explode(',', $_W['config']['setting']['founder']);
foreach($members as &$m) {
	$m['founder'] = in_array($m['uid'], $founders);
}

$usergroups = pdo_fetchall("SELECT id, name FROM ".tablename('users_group'), array(), 'id');
$settings = setting_load('register');
$settings = $settings['register'];
template('user/display');
