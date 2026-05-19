<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
$openid = $_W['openid'];
$dos = array('register', 'uc');
$do = in_array($do, $dos) ? $do : 'register';

$setting = uni_setting($_W['uniacid'], array('uc', 'passport'));
$uc_setting = $setting['uc'] ? $setting['uc'] : array();

$forward = url('mc');
if(!empty($_GPC['forward'])) {
	$forward = './index.php?' . base64_decode($_GPC['forward']) . '#wechat_redirect';
}
if (empty($setting['passport']['focusreg'])) {
	$reregister = false;
	if ($_W['member']['email'] == md5($_W['openid']).'@we7.cc') {
		$reregister = true;
	}
}

if(!$reregister && !empty($_W['member']) && (!empty($_W['member']['mobile']) || !empty($_W['member']['email']))) {
	header('location: ' . $forward);
	exit;
}

if($do == 'register') {
	if($_W['ispost'] && $_W['isajax']) {
		$post = $_GPC['__input'];
		$username = trim($post['username']);
		$password = trim($post['password']);
		$repassword = trim($post['repassword']);
		$repassword <> $password ? exit('两次密码输入不一致') : '';
		$sql = 'SELECT `uid` FROM ' . tablename('mc_members') . ' WHERE `uniacid`=:uniacid';
		$pars = array();
		$pars[':uniacid'] = $_W['uniacid'];
		if(preg_match('/^\d{11}$/', $username)) {
			$type = 'mobile';
			$sql .= ' AND `mobile`=:mobile';
			$pars[':mobile'] = $username;
		} elseif(preg_match("/^\w+([-+.]\w+)*@\w+([-.]\w+)*\.\w+([-.]\w+)*$/", $username)) {
			$type = 'email';
			$sql .= ' AND `email`=:email';
			$pars[':email'] = $username;
		} else {
			exit('您输入的用户名格式错误');
		}
		$user = pdo_fetch($sql, $pars);
		if(!empty($user)) {
			exit('该用户名已被注册，请输入其他用户名。');
		}
				if(!empty($_W['openid'])) {
			$map_fans = pdo_fetchcolumn('SELECT tag FROM ' . tablename('mc_mapping_fans') . ' WHERE uniacid = :uniacid AND openid = :openid', array(':uniacid' => $_W['uniacid'], ':openid' => $_W['openid']));
			if(!empty($map_fans)) {
				$map_fans = iunserializer($map_fans) ? iunserializer($map_fans) : array();
			}
		}
		
		if ($reregister) {
			$data = array(
				'salt' => random(8),
			);
			$type == 'email' ? ($data['email'] = $username) : ($data['mobile'] = $username);
			$type == 'email' ? '' : ($data['email'] = '');

			$data['password'] = md5($password . $data['salt'] . $_W['config']['setting']['authkey']);
			if(!empty($map_fans)) {
				$data['nickname'] = $map_fans['nickname'];
				$data['gender'] = $map_fans['sex'];
				$data['residecity'] = $map_fans['city'] ? $map_fans['city'] . '市' : '';
				$data['resideprovince'] = $map_fans['province'] ? $map_fans['province'] . '省' : '';
				$data['nationality'] = $map_fans['country'];
				$data['avatar'] = rtrim($map_fans['headimgurl'], '0') . 132;
			}
			pdo_update('mc_members', $data, array('uid' => $_W['member']['uid']));
			$user['uid'] = $_W['member']['uid'];
		} else {
			$default_groupid = pdo_fetchcolumn('SELECT groupid FROM ' .tablename('mc_groups') . ' WHERE uniacid = :uniacid AND isdefault = 1', array(':uniacid' => $_W['uniacid']));
			$data = array(
				'uniacid' => $_W['uniacid'], 
				'salt' => random(8),
				'groupid' => $default_groupid, 
				'createtime' => TIMESTAMP	
			);
			if(!empty($map_fans)) {
				$data['nickname'] = $map_fans['nickname'];
				$data['gender'] = $map_fans['sex'];
				$data['residecity'] = $map_fans['city'] ? $map_fans['city'] . '市' : '';
				$data['resideprovince'] = $map_fans['province'] ? $map_fans['province'] . '省' : '';
				$data['nationality'] = $map_fans['country'];
				$data['avatar'] = rtrim($map_fans['headimgurl'], '0') . 132;
			}
			$type == 'email' ? ($data['email'] = $username) : ($data['mobile'] = $username);
			$data['password'] = md5($password . $data['salt'] . $_W['config']['setting']['authkey']);
			pdo_insert('mc_members', $data);
			$user['uid'] = pdo_insertid();
		}
		if(_mc_login($user)) {
			exit('success');
		}
		exit('未知错误导致注册失败');
	}
	template('auth/register');
	exit;
}
