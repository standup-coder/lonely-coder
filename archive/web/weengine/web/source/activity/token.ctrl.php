<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
$dos = array('display', 'post','del', 'record', 'record-del');
$do = in_array($do, $dos) ? $do : 'display';

$creditnames = array();
$unisettings = uni_setting($uniacid, array('creditnames'));
foreach ($unisettings['creditnames'] as $key=>$credit) {
	if (!empty($credit['enabled'])) {
		$creditnames[$key] = $credit['title'];
	}
}

if($do == 'post') {
	global $_W, $_GPC;
	load()->func('tpl');
	$couponid = intval($_GPC['id']);
	$_W['page']['title'] = !empty($couponid) ? '代金券编辑 - 折扣券 - 会员营销' : '代金券添加 - 折扣券 - 会员营销';
	$item = pdo_fetch('SELECT * FROM ' . tablename('activity_coupon') . " WHERE uniacid = '{$_W['uniacid']}' AND couponid = '{$couponid}'");
		if(empty($item) || $couponid == 0) {
		$item['starttime'] = time();
		$item['endtime'] = time() + 6 * 86400;
	}
	$coupongroup = pdo_fetchall('SELECT groupid FROM ' . tablename('activity_coupon_allocation') . " WHERE uniacid = '{$_W['uniacid']}' AND couponid = '{$couponid}'");
	if(!empty($coupongroup)) {
		foreach($coupongroup as $cgroup) {
			$grouparr[] = $cgroup['groupid'];
		}
	}
	
		$group = pdo_fetchall('SELECT groupid,title FROM ' . tablename('mc_groups') . " WHERE uniacid = '{$_W['uniacid']}' ");
	if(!empty($grouparr)) {
		foreach($group as &$g){
			if(in_array($g['groupid'], $grouparr)) {
				$g['groupid_select'] = 1;
			}
		}
	}
		
	if(checksubmit('submit')) {
		$title = !empty($_GPC['title']) ? trim($_GPC['title']) : message('请输入代金券名称！');
		$condition = !empty($_GPC['condition']) ? trim($_GPC['condition']) : message('请输入满多少钱可用！');
		$discount = !empty($_GPC['discount']) ? trim($_GPC['discount']) : message('请输入抵消金额！');
		$groups = !empty($_GPC['group']) ? $_GPC['group'] : message('请选择可使用的会员组！');
		$thumb = !empty($_GPC['thumb']) ? $_GPC['thumb'] : message('请上传缩略图！');
		$description = !empty($_GPC['description']) ? trim($_GPC['description']) : message('请填写代金券说明！');
		
		$credittype = !empty($_GPC['credittype']) ? trim($_GPC['credittype']) : message('请选择积分类型！');
		$credit =  intval($_GPC['credit']);
		$limit = intval($_GPC['limit']) ? intval($_GPC['limit']) : message('每人限领数目必须为数字！');
		$amount = intval($_GPC['amount']) ? intval($_GPC['amount']) : message('代金券总数必须为数字！');
		$starttime = strtotime($_GPC['datelimit']['start']);
		$endtime = strtotime($_GPC['datelimit']['end']);
		if ($endtime == $starttime) {
			$endtime = $endtime + 86399;
		}
		$data = array(
			'uniacid' => $_W['uniacid'],
			'title' => $title,
			'type' => '2', 			'condition' => $condition,
			'discount' => $discount,
			'thumb' => $_GPC['thumb'],
			'description' => $description,
			'credittype' => $credittype,
			'credit' => $credit,
			'limit' => $limit,
			'amount' => $amount,
			'starttime' => $starttime,
			'endtime' => $endtime,
		);
		if ($couponid) {
			if(empty($item['couponsn'])) {
				$data['couponsn'] = 'AB' . $_W['uniacid'] . date('YmdHis');
			}
			pdo_update('activity_coupon', $data, array('uniacid' => $_W['uniacid'], 'couponid' => $couponid));
		} else {
			$data['couponsn'] = 'AB' . $_W['uniacid'] . date('YmdHis');
			pdo_insert('activity_coupon', $data);
			$couponid = pdo_insertid();
		}
		pdo_delete('activity_coupon_allocation', array('uniacid' => $_W['uniacid'], 'couponid' => $couponid));
		if($_GPC['group'] && $couponid) {
			foreach($_GPC['group'] as $gid) {
				$gid = intval($gid);
				$insert = array(
					'uniacid' => $_W['uniacid'],
					'couponid' => $couponid,
					'groupid' => $gid
				);
				pdo_insert('activity_coupon_allocation', $insert) ? '' : message('抱歉，代金券更新失败！', referer(), 'error');
				unset($insert);
			}
		}
		message('代金券更新成功！', url('activity/token/display'), 'success');
	}
}

if($do == 'display') {
	$_W['page']['title'] = '代金券管理 - 代金券 - 会员营销';
	$pindex = max(1, intval($_GPC['page']));
	$psize = 30;
	$condition = '';
	if(!empty($_GPC['keyword'])) {
		$condition .= " AND title LIKE '%{$_GPC['keyword']}%'";
	}
	if(!empty($_GPC['couponsn'])) {
		$condition .= " AND couponsn LIKE '%{$_GPC['couponsn']}%'";
	}
	if(intval($_GPC['groupid'])) {
		$total = pdo_fetchcolumn('SELECT COUNT(*) FROM ' . tablename('activity_coupon') . " WHERE uniacid = '{$_W['uniacid']}' AND type = 2 " . $condition . "  AND couponid IN (SELECT couponid FROM ".tablename('activity_coupon_allocation')." WHERE groupid = '{$_GPC['groupid']}')");
		$list = pdo_fetchall('SELECT * FROM ' . tablename('activity_coupon') . " WHERE uniacid = '{$_W['uniacid']}' AND type = 2 " . $condition . " AND couponid IN (SELECT couponid FROM ".tablename('activity_coupon_allocation')." WHERE groupid = '{$_GPC['groupid']}') ORDER BY couponid DESC LIMIT ".($pindex - 1) * $psize.','.$psize);
	} else {
		$total = pdo_fetchcolumn('SELECT COUNT(*) FROM ' . tablename('activity_coupon') . " WHERE uniacid = '{$_W['uniacid']}' AND type = 2" . $condition);
		$list = pdo_fetchall('SELECT * FROM ' . tablename('activity_coupon') . " WHERE uniacid = '{$_W['uniacid']}' AND type = 2 " . $condition . " ORDER BY couponid DESC LIMIT ".($pindex - 1) * $psize.','.$psize);
	}

		$groupall = pdo_fetchall('SELECT groupid,title FROM ' . tablename('mc_groups') . " WHERE uniacid = '{$_W['uniacid']}' ");
	foreach($list as &$li) {
		$group = pdo_fetchall('SELECT m.* FROM ' . tablename('activity_coupon_allocation') . " AS a LEFT JOIN ".tablename('mc_groups')." AS m ON a.groupid = m.groupid WHERE a.uniacid = '{$_W['uniacid']}' AND a.couponid = '{$li['couponid']}'");
		$li['group'] = $group;
	}
	foreach($list as &$li) {
		$li['thumb'] = tomedia($li['thumb']);
	}
	$pager = pagination($total, $pindex, $psize);
}

if($do == 'del') {
	$id = intval($_GPC['id']);
	$row = pdo_fetch("SELECT couponid FROM ".tablename('activity_coupon')." WHERE uniacid = '{$_W['uniacid']}' AND couponid = :couponid", array(':couponid' => $id));
	if (empty($row)) {
		message('抱歉，代金券不存在或是已经被删除！');
	}
	pdo_delete('activity_coupon_allocation', array('uniacid' => $_W['uniacid'],'couponid' => $id));
	pdo_delete('activity_coupon', array('uniacid' => $_W['uniacid'], 'couponid' => $id));
	message('代金券删除成功！',url('activity/token/display'), 'success');
}

if($do == 'record') {
	load()->func('tpl');
	$coupons = pdo_fetchall('SELECT couponid, title FROM ' . tablename('activity_coupon') . ' WHERE uniacid = :uniacid AND type = 2 ORDER BY couponid DESC', array(':uniacid' => $_W['uniacid']), 'couponid');
	$starttime = empty($_GPC['time']['start']) ? strtotime('-1 month') : strtotime($_GPC['time']['start']);
	$endtime = empty($_GPC['time']['end']) ? TIMESTAMP : strtotime($_GPC['time']['end']) + 86399;

	$where = " WHERE a.uniacid = {$_W['uniacid']} AND b.type = 2 AND a.granttime>=:starttime AND a.granttime<:endtime";
	$params = array(
		':starttime' => $starttime,
		':endtime' => $endtime,
	);
	$uid = intval($_GPC['uid']);
	if (!empty($uid)) {
		$where .= ' AND a.uid=:uid';
		$params[':uid'] = $uid;
	}
	$couponid = intval($_GPC['couponid']);
	if (!empty($couponid)) {
		$where .= " AND a.couponid = {$couponid}";
	}
	$pindex = max(1, intval($_GPC['page']));
	$psize = 20;

	$list = pdo_fetchall("SELECT a.*, b.title,b.thumb,b.discount FROM ".tablename('activity_coupon_record'). ' AS a LEFT JOIN ' . tablename('activity_coupon') . ' AS b ON a.couponid = b.couponid ' . " $where ORDER BY a.couponid DESC,a.recid DESC LIMIT ".($pindex - 1) * $psize.','.$psize, $params);
	$total = pdo_fetchcolumn("SELECT COUNT(*) FROM ".tablename('activity_coupon_record') . ' AS a LEFT JOIN ' . tablename('activity_coupon') . ' AS b ON a.couponid = b.couponid '. $where , $params);
	if(!empty($list)) {
		$uids = array();
		foreach ($list as $row) {
			$uids[] = $row['uid'];
		}
		load()->model('mc');
		$members = mc_fetch($uids, array('uid', 'nickname'));
		foreach ($list as &$row) {
			$row['nickname'] = $members[$row['uid']]['nickname'];
			$row['thumb'] = tomedia($row['thumb']);
		}
	}
	$pager = pagination($total, $pindex, $psize);
}
if($do == 'record-del') {
	$id = intval($_GPC['id']);
	if(empty($id)) {
		message('没有要删除的记录', '', 'error');
	}
	pdo_delete('activity_coupon_record', array('recid' => $id));
	message('删除兑换记录成功', '', 'success');
}
template('activity/token');