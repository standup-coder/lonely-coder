<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
if(!empty($_GPC['multiid'])) {
	define('ACTIVE_FRAME_URL', url('site/multi/display'));
}

if($action == 'cover') {
	$dos = array('site', 'mc', 'card','module');
	$do = in_array($do, $dos) ? $do : 'module';
	if($do != 'module' && $do != 'card') {
		define('FRAME', $do);
	}
	if($do == 'card') {
		define('FRAME', 'mc');		
	}
} elseif($action == 'reply') {
	$m = $_GPC['m'];
	if(in_array($m, array('basic', 'news', 'music', 'userapi'))) {
		define('FRAME', 'platform');
	}
} else {
	define('FRAME', 'platform');
}

$frames = buildframes(array(FRAME));
$frames = $frames[FRAME];
