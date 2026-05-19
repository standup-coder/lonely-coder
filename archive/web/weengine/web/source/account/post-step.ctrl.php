<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */

defined('IN_IA') or exit('Access Denied');
load()->func('tpl');

$id = $uniacid = intval($_GPC['uniacid']);
if(!empty($id)) {
	$state = uni_permission($uid, $id);
	if($state != 'founder' && $state != 'manager') {
		message('没有该公众号操作权限！');
	}
} else {
	if(empty($_W['isfounder']) && is_error($permission = uni_create_permission($_W['uid'], 1))) {
		message($permission['message'], '' , 'error');
		if(is_error($permission = uni_create_permission($_W['uid'], 2))) {
			message($permission['message'], '' , 'error');
		}
	}
}


$step = intval($_GPC['step']) ? intval($_GPC['step']) : 1;
if($step == 1) {

} elseif($step == 2) {
	if(checksubmit('submit')) {
		$name = trim($_GPC['name']) ? trim($_GPC['name']) : message('抱歉，名称为必填项请返回填写！');
		$data = array(
			'name' => trim($_GPC['name']),
			'description' => ($_GPC['description']),
			'groupid' => 0
		);
		$state = pdo_insert('uni_account', $data);
		if(!$state) message('添加公众号失败');
		$uniacid = pdo_insertid();
				$template = pdo_fetch('SELECT id,title FROM ' . tablename('site_templates') . " WHERE name = 'default'");
		$styles['uniacid'] = $uniacid;
		$styles['templateid'] = $template['id'];
		$styles['name'] = $template['title'] . '_' . random(4);
		pdo_insert('site_styles', $styles);
		$styleid = pdo_insertid();
				$multi['uniacid'] = $uniacid;
		$multi['title'] = $data['name'];
		$multi['quickmenu'] = iserializer(array('template' => 'default', 'enablemodule' => array()));
		$multi['styleid'] = $styleid;
		pdo_insert('site_multi', $multi);
		$multi_id = pdo_insertid();
		
		$unisettings['creditnames'] = array('credit1' => array('title' => '积分', 'enabled' => 1), 'credit2' => array('title' => '余额', 'enabled' => 1));
		$unisettings['creditnames'] = iserializer($unisettings['creditnames']);
		$unisettings['creditbehaviors'] = array('activity' => 'credit1', 'currency' => 'credit2');
		$unisettings['creditbehaviors'] = iserializer($unisettings['creditbehaviors']);
		$unisettings['uniacid'] = $uniacid;
		$unisettings['default_site'] = $multi_id; 		$unisettings['sync'] = iserializer(array('switch' => 0, 'acid' => ''));
		pdo_insert('uni_settings', $unisettings);
		
		pdo_insert('mc_groups', array('uniacid' => $uniacid, 'title' => '默认会员组', 'isdefault' => 1));
		$account_users = array('uniacid' => $uniacid, 'uid' => $_W['uid'], 'role' => 'manager');
		pdo_insert('uni_account_users', $account_users);
		
		load()->model('module');
		module_build_privileges();
	}
	
} elseif($step == 3) {
	$uniacid = intval($_GPC['uniacid']);
	if(checksubmit('submit')) {
		load()->func('file');
		if (intval($_GPC['type']) == '2') {
			$type = 'yixin';
		} elseif (intval($_GPC['type']) == '3') {
			$type = 'alipay';
		} else {
			$type = 'wechat';
		}
		$username = trim($_GPC['wxusername']);
		$password = md5($_GPC['wxpassword']);
		if(!empty($username) && !empty($password)) {
			if ($type == 'wechat') {
				$loginstatus = account_weixin_login($username, $password, $_GPC['verify']);
				if(is_error($loginstatus)) {
					message($loginstatus['message'], url('account/post-step', array('uniacid' => $uniacid, 'step' => 2)), 'error');
				}
				$basicinfo = account_weixin_basic($username);
			} elseif ($_GPC['type'] == 'yixin') {
				$loginstatus = account_yixin_login($username, $password, $_GPC['verify']);
				if(is_error($loginstatus)) {
					message($loginstatus['message'], url('account/post-step', array('uniacid' => $uniacid, 'step' => 2)), 'error');
				}
				$basicinfo = account_yixin_basic($username);
			}
			if (empty($basicinfo['name'])) {
				message('一键获取信息失败,请手动设置公众号信息！', url('account/post-step/', array('uniacid' => $uniacid, 'step' => 3)), 'error');
			}
			$account['username'] = $_GPC['wxusername'];
			$account['password'] = md5($_GPC['wxpassword']);
			$account['lastupdate'] = TIMESTAMP;
			$account['name'] = $basicinfo['name'];
			$account['account'] = $basicinfo['account'];
			$account['original'] = $basicinfo['original'];
			$account['signature'] = $basicinfo['signature'];
			$account['key'] = $basicinfo['key'];
			$account['secret'] = $basicinfo['secret'];
			$account['type'] = intval($_GPC['type']);
			$account['level'] = $basicinfo['level'];
		} else {
			message('请填写公众平台用户名和密码', url('account/post-step', array('uniacid' => $uniacid, 'step' => 2)), 'error');
		}
		$acid = account_create($uniacid, $account);
		if(is_error($acid)) {
			message('添加公众号信息失败', '', url('account/post-step/', array('uniacid' => $uniacid, 'step' => 2), 'error'));
		}
				if (!empty($basicinfo['headimg'])) {
			file_write('headimg_'.$acid.'.jpg', $basicinfo['headimg']);
		}
		if (!empty($basicinfo['qrcode'])) {
			file_write('qrcode_'.$acid.'.jpg', $basicinfo['qrcode']);
		}
	}
	
	if(!empty($acid)) {
		$account = account_fetch($acid);
	}
} elseif($step == 4) {
	$uniacid = intval($_GPC['uniacid']);
	$acid = intval($_GPC['acid']);
	$account = account_fetch($acid);
	$flag = intval($_GPC['flag']);
	
	if(checksubmit('submit') && $flag == 1) {
		load()->func('file');
		$update['name'] = $_GPC['name'];
		if(empty($update['name'])) {
			message('公众号名称必须填写');
		}
		$update['account'] = $_GPC['account'];
		$update['level'] = intval($_GPC['level']);
		$update['key'] = $_GPC['key'];
		$update['secret'] = $_GPC['secret'];
		$update['type'] = intval($_GPC['type']);
		if(empty($account)) {
			$acid = account_create($uniacid, $update);
			if(is_error($acid)) {
				message('添加公众号信息失败', '', url('account/post-step/', array('uniacid' => intval($_GPC['uniacid']), 'step' => 3), 'error'));
			}
			if (!empty($_FILES['qrcode']['tmp_name'])) {
				$_W['uploadsetting'] = array();
				$_W['uploadsetting']['image']['folder'] = '';
				$_W['uploadsetting']['image']['extentions'] = array('jpg');
				$_W['uploadsetting']['image']['limit'] = $_W['config']['upload']['image']['limit'];
				$upload = file_upload($_FILES['qrcode'], 'image', "qrcode_{$acid}");
			}
			if (!empty($_FILES['headimg']['tmp_name'])) {
				$_W['uploadsetting'] = array();
				$_W['uploadsetting']['image']['folder'] = '';
				$_W['uploadsetting']['image']['extentions'] = array('jpg');
				$_W['uploadsetting']['image']['limit'] = $_W['config']['upload']['image']['limit'];
				$upload = file_upload($_FILES['headimg'], 'image', "headimg_{$acid}");
			}
		} else {
			pdo_update('account', array('type' => intval($_GPC['type']), 'hash' => random(8)), array('acid' => $acid, 'uniacid' => $uniacid));
	
			if($update['type'] == 1) {
				unset($update['type']);
				pdo_update('account_wechats', $update, array('acid' => $acid, 'uniacid' => $uniacid));
			} else if($update['type'] == 2) {
				unset($update['type']);
				pdo_update('account_yixin', $update, array('acid' => $acid, 'uniacid' => $uniacid));
	
			}else if($update['type'] == 3) {
				unset($update['type']);
				pdo_update('account_alipay', $update, array('acid' => $acid, 'uniacid' => $uniacid));
	
			}
			
			if (!empty($_FILES['qrcode']['tmp_name'])) {
				$_W['uploadsetting'] = array();
				$_W['uploadsetting']['image']['folder'] = '';
				$_W['uploadsetting']['image']['extentions'] = array('jpg');
				$_W['uploadsetting']['image']['limit'] = $_W['config']['upload']['image']['limit'];
				$upload = file_upload($_FILES['qrcode'], 'image', "qrcode_{$acid}");
			}
			if (!empty($_FILES['headimg']['tmp_name'])) {
				$_W['uploadsetting'] = array();
				$_W['uploadsetting']['image']['folder'] = '';
				$_W['uploadsetting']['image']['extentions'] = array('jpg');
				$_W['uploadsetting']['image']['limit'] = $_W['config']['upload']['image']['limit'];
				$upload = file_upload($_FILES['headimg'], 'image', "headimg_{$acid}");
			}
		}
	}

		if (empty($_W['isfounder'])) {
		$group = pdo_fetch("SELECT * FROM ".tablename('users_group')." WHERE id = '{$_W['user']['groupid']}'");
		$group['package'] = uni_groups((array)iunserializer($group['package']));
	} else {
		$group['package'] = uni_groups();
	}
	$allow_group = array_keys($group['package']);
	$allow_group[] = 0;
	if(!empty($_W['isfounder'])) {
		$allow_group[] = -1;
	}
	
	if(!$acid) {
	message('未填写公众号信息', '', url('account/post-step/', array('uniacid' => intval($_GPC['uniacid']), 'step' => 3), 'error'));
	}
	
	if(checksubmit('submit') && $flag == 2) {
		$groupid = intval($_GPC['groupid']);
		
		if(!in_array($groupid, $allow_group)) {
			message('您所在的用户组没有使用该服务套餐的权限');
		}
		pdo_update('uni_account', array('groupid' => $groupid), array('uniacid' => $uniacid));
		
		if($_GPC['isexpire'] == '1') {
			strtotime($_GPC['endtime']) > TIMESTAMP ? '' : message('服务套餐过期时间必须大于当前时间', '', 'error');
			$uniaccount['groupdata'] = iserializer(array('isexpire' => 1, 'oldgroupid' => '', 'endtime' => strtotime($_GPC['endtime'])));
		} else {
			$uniaccount['groupdata'] = iserializer(array('isexpire' => 0, 'oldgroupid' => '', 'endtime' => TIMESTAMP));
		}
				$notify['sms']['balance'] = intval($_GPC['balance']);
		$notify['sms']['signature'] = trim($_GPC['signature']);
		$uniaccount['notify'] = iserializer($notify);
				$uniaccount['bootstrap'] = trim($_GPC['bootstrap']);
		
		pdo_update('uni_settings', $uniaccount, array('uniacid' => $uniacid));
		header('Location:' . url('account/post-step/', array('uniacid' => $uniacid, 'step' => 5, 'acid' => $acid)));
		exit;
	}	
	
} elseif($step == 5) {
	$uniacid = intval($_GPC['uniacid']);
	$acid = intval($_GPC['acid']);
	$isexist = pdo_fetch('SELECT uniacid FROM ' . tablename('uni_account') . ' WHERE uniacid = ' . $uniacid);
	if(empty($isexist)) {
		message('非法访问');
	}
	$account = account_fetch($acid);
}

template('account/post-step');
