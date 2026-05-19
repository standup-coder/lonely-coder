<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
if(!empty($_GPC['f']) && $_GPC['f'] == 'multi') {
	define('ACTIVE_FRAME_URL', url('site/multi/display'));
}
if(!empty($_GPC['styleid'])) {
	define('ACTIVE_FRAME_URL', url('site/style/styles'));
}

if($action != 'entry' && $action != 'nav') {
	define('FRAME', 'site');
}
$frames = buildframes(array(FRAME));
$frames = $frames[FRAME];
