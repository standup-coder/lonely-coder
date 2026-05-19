<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
require './framework/bootstrap.inc.php';
$agent = strtolower($_SERVER['HTTP_USER_AGENT']);
if(strexists($agent, 'mobile')) {
	$sql = 'SELECT `uniacid` FROM ' . tablename('cover_reply') . " WHERE `module`='site' ORDER BY `id` LIMIT 1";
	$uniacid = pdo_fetchcolumn($sql);
	if(!empty($uniacid)) {
		header('location: ./app/index.php?i=' . $uniacid . '&c=home');
	}
} else {
	header('location: ./web/index.php');
}