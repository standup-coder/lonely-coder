<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
$_W['page']['title'] = '编辑用户 - 用户管理 - 用户管理';
load()->model('setting');
load()->func('tpl');
$uid = intval($_GPC['uid']);
$m = array();
$m['uid'] = $uid;
$member = user_single($m);
$founders = explode(',', $_W['config']['setting']['founder']);
if(empty($member) || in_array($m['uid'], $founders)) {
	message('访问错误.', url('user/display'), 'error');
}
$do = $_GPC['do'];
$dos = array('delete', 'edit');
$do = in_array($do, $dos) ? $do: 'edit';

if ($do == 'edit') {
	$extendfields = pdo_fetchall("SELECT field, title, description, required FROM ".tablename('profile_fields')." WHERE available = '1' AND showinregister = '1'");
	if(checksubmit('profile_submit')) {
		load()->model('user');
		$nMember = array();
		$nMember['uid'] = $uid;
		$nMember['password'] = trim($_GPC['password']);
		$nMember['salt'] = $member['salt'];
		$nMember['groupid'] = intval($_GPC['groupid']) ? intval($_GPC['groupid']) : message('请选择所属用户组');
		if(!empty($nMember['password']) && istrlen($nMember['password']) < 8) {
			message('必须输入密码，且密码长度不得低于8位。');
		}
		$nMember['remark'] = $_GPC['remark'];
		user_update($nMember);
		
		if (!empty($_GPC['birth'])) {
			$profile['birthyear'] = $_GPC['birth']['year'];
			$profile['birthmonth'] = $_GPC['birth']['month'];
			$profile['birthday'] = $_GPC['birth']['day'];
		}
		if (!empty($_GPC['reside'])) {
			$profile['resideprovince'] = $_GPC['reside']['province'];
			$profile['residecity'] = $_GPC['reside']['city'];
			$profile['residedist'] = $_GPC['reside']['district'];
		}
		
		if (!empty($extendfields)) {
			foreach ($extendfields as $row) {
				if(!in_array($row['field'], array('profile','resideprovince','birthyear'))) {
					$profile[$row['field']] = $_GPC[$row['field']];
				}
			}
			
			if (!empty($profile)) {
				$exists = pdo_fetchcolumn("SELECT uid FROM ".tablename('users_profile')." WHERE uid = :uid", array(':uid' => $uid));
				if (!empty($exists)) {
					pdo_update('users_profile', $profile, array('uid' => $uid));
				} else {
					$profile['uid'] = $uid;
					pdo_insert('users_profile', $profile);
				}
			}
		}
		message('保存用户资料成功！', 'refresh');
	}
	
	$member['profile'] = pdo_fetch("SELECT * FROM ".tablename('users_profile')." WHERE uid = :uid", array(':uid' => $uid));
	$member['profile']['reside'] = array(
		'province' => $member['profile']['resideprovince'],
		'city' => $member['profile']['residecity'],
		'district' => $member['profile']['residedist'],
	);
	unset($member['profile']['resideprovince']);
	unset($member['profile']['residecity']);
	unset($member['profile']['residedist']);
	$member['profile']['birth'] = array(
		'year' => $member['profile']['birthyear'],
		'month' => $member['profile']['birthmonth'],
		'day' => $member['profile']['birthday'],
	);
	unset($member['profile']['birthyear']);
	unset($member['profile']['birthmonth']);
	unset($member['profile']['birthday']);
	
	$groups = pdo_fetchall("SELECT id, name FROM ".tablename('users_group')." ORDER BY id ASC");
	
	template('user/edit');
	exit;
}

if($do == 'delete') {
	if($_W['ispost'] && $_W['isajax']) {
		$founders = explode(',', $_W['config']['setting']['founder']);
		if(in_array($uid, $founders)) {
			exit('管理员用户不能删除.');
		}
		$member = array();
		$member['uid'] = $uid;
		if(pdo_delete('users', $member) === 1) {
						pdo_delete('uni_account_users', array('uid' => $uid));
			pdo_delete('users_profile', array('uid' => $uid));
			pdo_delete('solution_acl', array('uid' => $uid));
			exit('success');
		}
	}
}
