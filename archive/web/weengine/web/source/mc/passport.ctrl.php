<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
$dos = array('passport', 'oauth', 'sync');
$do = in_array($do, $dos) ? $do : 'passport';
if($do == 'passport') {
	$_W['page']['title'] = '会员中心参数 - 会员中心选项 - 会员中心';
	$uc = pdo_fetch("SELECT `uc`,`passport` FROM ".tablename('uni_settings') . " WHERE uniacid = :uniacid", array(':uniacid' => $_W['uniacid']));
	$passport = @iunserializer($uc['passport']);
	if(!is_array($passport)) {
		$passport = array();
	}
	
	if(checksubmit('submit')) {
		$rec = array();
		$passport = array();
		$passport['focusreg'] = intval($_GPC['passport']['focusreg']);
		$passport['item'] = $_GPC['passport']['item'] == 'mobile' ? 'mobile' : 'email';
		$passport['type'] = $_GPC['passport']['type'];
		$passport['type'] = in_array($passport['type'], array('code', 'password', 'hybird')) ? $passport['type'] : 'password';
		$rec['passport'] = iserializer($passport);
		$row = pdo_fetch("SELECT uniacid FROM ".tablename('uni_settings') . " WHERE uniacid = :wid LIMIT 1", array(':wid' => intval($_W['uniacid'])));
		if(!empty($row)) {
			pdo_update('uni_settings', $rec, array('uniacid' => intval($_W['uniacid'])));
		}else {
			pdo_insert('uni_settings', $rec);
		}
		message('设置成功！', referer(), 'success');
	}
}

if($do == 'oauth') {
		$_W['page']['title'] = '公众平台oAuth选项 - 会员中心';
	$accountlist = uni_accounts($_W['uniacid']);
	$data = array();
	if(!empty($accountlist)) {
		foreach($accountlist as $list) {
			if($list['level'] == 2) {
				$data[] = $list;
			}
		}
	}
		$oauth = pdo_fetchcolumn('SELECT `oauth` FROM '.tablename('uni_settings').' WHERE `uniacid` = :uniacid LIMIT 1',array(':uniacid' => $_W['uniacid']));
	$oauth = iunserializer($oauth) ? iunserializer($oauth) : array('status' => 0, 'account' => '');
	if(checksubmit('submit')) {
		($_GPC['oauth']['status'] == 1) && empty($_GPC['oauth']['account']) ? message('开启公众平台oAuth后,必须选择公众号', '' ,'error') : ''; 
		$post = iserializer($_GPC['oauth']);
		pdo_update('uni_settings', array('oauth' => $post), array('uniacid' => $_W['uniacid']));
		message('设置公众平台oAuth成功', referer() ,'success');
	}
}

if($do == 'sync') {
	$_W['page']['title'] = '更新粉丝信息 - 公众号选项';
	$setting = uni_setting($_W['uniacid'], array('sync'));
	$sync = $setting['sync'];
	if(!is_array($sync)) {
		$sync = array();
	}
	$accs = uni_accounts();
	$accounts = array();
	if(!empty($accs)) {
		foreach($accs as $acc) {
			if($acc['type'] == '1' && $acc['level'] == '2') {
				$accounts[$acc['acid']] = array_elements(array('name', 'acid'), $acc);
			}
		}
	}
	if(checksubmit('submit')) {
		if($_GPC['sync']['switch'] == 1) {
			if(!in_array($_GPC['sync']['acid'], array_keys($accounts))) message('选择公众号出现错误', '', 'error');
		}		
		pdo_update('uni_settings', array('sync' => iserializer($_GPC['sync'])), array('uniacid' => $_W['uniacid']));
		message('更新设置成功', referer(),  'success');
	}
}
template('mc/passport');