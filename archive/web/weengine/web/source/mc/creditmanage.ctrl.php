<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
$dos = array('display', 'manage','modal','credit_record');
$do = in_array($do, $dos) ? $do : 'display';

$creditnames = uni_setting($_W['uniacid'], array('creditnames'));
if($creditnames) {
	foreach($creditnames['creditnames'] as $index=>$creditname) {
		if($creditname['enabled'] == 0) {
			unset($creditnames['creditnames'][$index]);
		}
	}
	$select_credit = implode(', ', array_keys($creditnames['creditnames']));
} else {
	$select_credit = '';
}

if($do == 'display') {
	$_W['page']['title'] = '积分列表 - 会员积分管理 - 会员中心';
	
	$wheresql = ' WHERE uniacid = :uniacid ';
	$type = intval($_GPC['type']);
	$keyword = trim($_GPC['keyword']);
	if($type == 1) {
		$keyword = intval($_GPC['keyword']);
		$wheresql .= $keyword ? ' AND uid = ' . $keyword : '';
	} elseif($type == 2) {
		$wheresql .= $keyword ? " AND mobile LIKE '%{$keyword}%' " : '';
	} elseif($type == 3) {
		$wheresql .= $keyword ? " AND realname LIKE '%{$keyword}%' " : '';
	}
	$pindex = max(1, intval($_GPC['page']));
	$psize = 20;
	
	$total = pdo_fetchcolumn("SELECT COUNT(*) FROM " . tablename('mc_members') . $wheresql, array(':uniacid' => $_W['uniacid']));
	$list = pdo_fetchall("SELECT uid, uniacid, email, realname, mobile, {$select_credit} FROM " . tablename('mc_members') . $wheresql . ' ORDER BY uid DESC LIMIT ' . ($pindex - 1) * $psize .',' . $psize, array(':uniacid' => $_W['uniacid']));
	$pager = pagination($total, $pindex, $psize);
	if(count($list) == 1 && $list[0]['uid'] && !empty($keyword)) {
		$status = 1;
		$uid = $list[0]['uid'];
	} else {
		$status = 0;
	}
}
if($do == 'manage') {
	load()->model('mc');
	$uid = intval($_GPC['uid']);
	if($uid) {
		foreach($creditnames['creditnames'] as $index=>$creditname) {
			if(($_GPC[$index . '_type'] == 1 || $_GPC[$index . '_type'] == 2) && $_GPC[$index . '_value']) {
				$value = $_GPC[$index . '_type'] == 1 ? $_GPC[$index . '_value'] : - $_GPC[$index . '_value'];
				$return = mc_credit_update($uid, $index, $value, array($_W['uid'], trim($_GPC['remark'])));
				if(is_error($return)) {
					message($return['message']);
				}
			} else {
				continue;
			}
		}
		message('会员积分操作成功', url('mc/creditmanage/display'));
	} else {
		message('未找到指定用户', url('mc/creditmanage/display'), 'error');
	}
}
if($do == 'modal') {
	if($_W['isajax']) {
		$uid = intval($_GPC['uid']);
		$data = pdo_fetch("SELECT uid, realname, email, mobile, uniacid, {$select_credit} FROM " . tablename('mc_members') . ' WHERE uid = :uid AND uniacid = :uniacid', array(':uniacid' => $_W['uniacid'], ':uid' => $uid));
		$data ? template('mc/modal') : exit('dataerr');
		exit();
	}
}
if($do == 'credit_record') {
	$uid = intval($_GPC['uid']);
	$credits = array_keys($creditnames['creditnames']);
	$type = trim($_GPC['type']) ? trim($_GPC['type']) : $credits[0];
	
	$pindex = max(1, intval($_GPC['page']));
	$psize = 50;
	$total = pdo_fetchcolumn("SELECT COUNT(*) FROM " . tablename('mc_credits_record') . ' WHERE uid = :uid AND uniacid = :uniacid AND credittype = :credittype ', array(':uniacid' => $_W['uniacid'], ':uid' => $uid, 'credittype' => $type));
	$data = pdo_fetchall("SELECT r.*, u.username FROM " . tablename('mc_credits_record') . ' AS r LEFT JOIN ' .tablename('users') . ' AS u ON r.operator = u.uid ' . ' WHERE r.uid = :uid AND r.uniacid = :uniacid AND r.credittype = :credittype ORDER BY id DESC LIMIT ' . ($pindex - 1) * $psize .',' . $psize, array(':uniacid' => $_W['uniacid'], ':uid' => $uid, 'credittype' => $type));
	$pager = pagination($total, $pindex, $psize);
	template('mc/credit_record');
	exit;
}

template('mc/creditmanage');
