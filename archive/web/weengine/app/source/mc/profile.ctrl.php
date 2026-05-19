<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
load()->model('app');;

$title = $_W['account']['name'] . '微站';

$navs = app_navs('profile');
load()->func('tpl');
$profile = mc_fetch($_W['member']['uid']);
if(!empty($_W['openid'])) {
	$map_fans = pdo_fetchcolumn('SELECT tag FROM ' . tablename('mc_mapping_fans') . ' WHERE uniacid = :uniacid AND openid = :openid', array(':uniacid' => $_W['uniacid'], ':openid' => $_W['openid']));
	if(!empty($map_fans)) {
		$map_fans = iunserializer($map_fans) ? iunserializer($map_fans) : array();
		if(!empty($map_fans)) {
						empty($profile['nickname']) ? ($data['nickname'] = $map_fans['nickname']) : '';
			empty($profile['gender']) ? ($data['gender'] = $map_fans['sex']) : '';
			empty($profile['residecity']) ? ($data['residecity'] = ($map_fans['city']) ? $map_fans['city'] . '市' : '') : '';
			empty($profile['resideprovince']) ? ($data['resideprovince'] = ($map_fans['province']) ? $map_fans['province'] . '省' : '') : '';
			empty($profile['nationality']) ? ($data['nationality'] = $map_fans['country']) : '';
			empty($profile['avatar']) ? ($data['avatar'] = rtrim($map_fans['headimgurl'], '0') . 132) : '';
			if(!empty($data)) {
				mc_update($_W['member']['uid'], $data);
			}
		}
	}
}
$profile = mc_fetch($_W['member']['uid']);

if (checksubmit('submit')) {
	if (!empty($_GPC)) {
		$_GPC['createtime'] = TIMESTAMP;
		foreach ($_GPC as $field => $value) {
			if (!isset($value) || in_array($field, array('uid','act', 'name', 'token', 'submit', 'session'))) {
				unset($_GPC[$field]);
				continue;
			}
		}
		$_GPC['birthyear'] = $_GPC['birth']['year'];
		$_GPC['birthmonth'] = $_GPC['birth']['month'];
		$_GPC['birthday'] = $_GPC['birth']['day'];
		$_GPC['resideprovince'] = $_GPC['reside']['province'];
		$_GPC['residecity'] = $_GPC['reside']['city'];
		$_GPC['residedist'] = $_GPC['reside']['district'];
		mc_update($_W['member']['uid'], $_GPC);
	}
	message('更新资料成功！', referer(), 'success');
}
template('mc/profile');
