<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');

function user_register($member) {
	$member['salt'] = random(8);
	$member['joindate'] = TIMESTAMP;
	$member['password'] = user_hash($member['password'], $member['salt']);
	empty($member['status']) && $member['status'] = 0;
	$member['joinip'] = CLIENT_IP;
	$member['lastvisit'] = TIMESTAMP;
	$member['lastip'] = CLIENT_IP;
	$result = pdo_insert('users', $member);
	if($result) {
		if(empty($member['uid'])) {
			$member['uid'] = pdo_insertid();
		}
	}
	return $member['uid'];
}


function user_check($member) {
	$sql = 'SELECT `password`,`salt` FROM ' . tablename('users') . " WHERE 1";
	$params = array();
	if(!empty($member['uid'])) {
		$sql .= ' AND `uid`=:uid';
		$params[':uid'] = intval($member['uid']);
	}
	if(!empty($member['username'])) {
		$sql .= ' AND `username`=:username';
		$params[':username'] = $member['username'];
	}
	if(!empty($member['status'])) {
		$sql .= " AND `status`=:status";
		$params[':status'] = intval($member['status']);
	}
	$sql .= " LIMIT 1";
	$record = pdo_fetch($sql, $params);
	if(!$record || empty($record['password']) || empty($record['salt'])) {
		return false;
	}
	if(!empty($member['password'])) {
		$password = user_hash($member['password'], $record['salt']);
		return $password == $record['password'];
	}
	return true;
}


function user_single($member) {
	$sql = 'SELECT * FROM ' . tablename('users') . " WHERE 1";
	$params = array();
	if(!empty($member['uid'])) {
		$sql .= ' AND `uid`=:uid';
		$params[':uid'] = intval($member['uid']);
	}
	if(!empty($member['username'])) {
		$sql .= ' AND `username`=:username';
		$params[':username'] = $member['username'];
	}
	if(!empty($member['email'])) {
		$sql .= ' AND `email`=:email';
		$params[':email'] = $member['email'];
	}
	if(!empty($member['status'])) {
		$sql .= " AND `status`=:status";
		$params[':status'] = intval($member['status']);
	}
	$sql .= " LIMIT 1";
	$record = pdo_fetch($sql, $params);
	if(!$record) {
		return false;
	}
	if(!empty($member['password'])) {
		$password = user_hash($member['password'], $record['salt']);
		if($password != $record['password']) {
			return false;
		}
	}
	return $record;
}


function user_update($member) {
	if(empty($member['uid'])) {
		return false;
	}
	$params = array();
	if($member['password']) {
		$params['password'] = user_hash($member['password'], $member['salt']);
	}
	if($member['lastvisit']) {
		$params['lastvisit'] = (strlen($member['lastvisit']) == 10) ? $member['lastvisit'] : strtotime($member['lastvisit']);
	}
	if($member['lastip']) {
		$params['lastip'] = $member['lastip'];
	}
	if(isset($member['joinip'])) {
		$params['joinip'] = $member['joinip'];
	}
	if(isset($member['remark'])) {
		$params['remark'] = $member['remark'];
	}
	if(isset($member['status'])) {
		$params['status'] = $member['status'];
	}
	if (isset($member['groupid'])) {
		$params['groupid'] = $member['groupid'];
	}
	if(empty($params)) {
		return false;
	}

	return pdo_update('users', $params, array('uid' => intval($member['uid'])));
}


function user_hash($input, $salt) {
	global $_W;
	$input = "{$input}-{$salt}-{$_W['config']['setting']['authkey']}";
	return sha1($input);
}

function user_level() {
	static $level = array(
		'-3' => '锁定用户',
		'-2' => '禁止访问',
		'-1' => '禁止发言',
		'0' => '普通会员',
		'1' => '管理员',
	);
	return $level;
}
