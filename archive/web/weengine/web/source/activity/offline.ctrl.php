<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */

defined('IN_IA') or exit('Access Denied');

$dos = array('introduce', 'clerk');
$do = in_array($do, $dos) ? $do : 'introduce';

if($do == 'introduce') {
	$_W['page']['title'] = '功能说明 - 门店营销参数 - 会员营销';
	template('activity/offline');
}

if($do == 'clerk') {
	$_W['page']['title'] = '店员管理 - 门店营销参数 - 会员营销';
	if (checksubmit('submit')) {
		if (!empty($_GPC['title'])) {
			foreach ($_GPC['title'] as $index => $row) {
				$data = array(
					'name' => $_GPC['title'][$index],
				);
				if (!empty($_GPC['password'][$index])) {
					$data['password'] = $_GPC['password'][$index];
				}
				if(!empty($data['name'])) {
					$row = pdo_fetch("SELECT id FROM ".tablename('activity_coupon_password')." WHERE name = :name AND password = :password LIMIT 1",array(':name' => $data['name'],':password' => $data['password']));
					if(empty($row)) {
						pdo_update('activity_coupon_password', $data, array('id' => $index));
					}
					unset($row);
				}
			}
		}
		
		if (!empty($_GPC['title-new'])) {
			foreach ($_GPC['title-new'] as $index => $row) {
				$data = array(
						'uniacid' => $_W['uniacid'],
						'name' => $_GPC['title-new'][$index],
						'password' => $_GPC['password-new'][$index],
				);
				if(!empty($data['name']) && !empty($data['password'])) {
					$row = pdo_fetch("SELECT id FROM ".tablename('activity_coupon_password')." WHERE name = :name AND password = :password LIMIT 1",array(':name' => $data['name'],':password' => $data['password']));
					if(empty($row)) {
						pdo_insert('activity_coupon_password', $data);
					}
					unset($row);
				}
			}
		}
		
		if (!empty($_GPC['delete'])) {
			pdo_query("DELETE FROM ".tablename('activity_coupon_password')." WHERE id IN (".implode(',', $_GPC['delete']).")");
		}
		
		$pwd = pdo_fetchall('SELECT password FROM '.tablename('activity_coupon_password').' WHERE uniacid=:uniacid', array(':uniacid'=>$_W['uniacid']));
		$count = count($pwd);
		$arr = array();
		foreach ($pwd as $row) {
			$arr[] = $row['password'];
		}
		$count2 = count(array_unique($arr));
		if ($count != $count2) {
			message('消费密码有重复，请修改后店员的消费密码！', referer(), 'error');
		} else {
			message('消费密码更新成功！', referer(), 'success');
		}
	}
	
	$list = pdo_fetchall("SELECT * FROM ".tablename('activity_coupon_password')." WHERE uniacid = :uniacid", array(':uniacid' => $_W['uniacid']));
	template('activity/clerk');
}
