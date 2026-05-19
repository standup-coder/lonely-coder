<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');

load()->func('cache.' . $_W['config']['setting']['cache']);


function cache_load($key, $unserialize = false) {
	global $_W;
	if(substr($key, -1) == ':') {
		$data = cache_search($key);
		foreach($data as $k => $v) {
			$tmp = &cache_global($k);
			$tmp = $unserialize ? iunserializer($v) : $v;
		}
		return cache_global($key);
	} else {
		$data = cache_read($key);
		if ($key == 'setting') {
			$_W['setting'] = $data;
			return $_W['setting'];
		} elseif ($key == 'modules') {
			$_W['modules'] = $data;
			return $_W['modules'];
		} else {
			$tmp = &cache_global($key);
			$tmp = $unserialize ? iunserializer($data) : $data;
			return $unserialize ? iunserializer($data) : $data;
		}
	}
}

function &cache_global($key) {
	global $_W;
	$keys = explode(':', $key);
	$tmp = &$_W['cache'];
	foreach($keys as $v) {
		if(empty($v)) {
			continue;
		}
		$tmp = &$tmp[$v];
	}
	return $tmp;
}
