<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
if (!empty($_W['uid'])) {
	header('Location: '.url('account/display'));
	exit;
}

load()->model('setting');
$settings = setting_load(array('copyright', 'register'));
$copyright = $settings['copyright'];

if (isset($copyright['showhomepage']) && empty($copyright['showhomepage'])) {
	header("Location: ".url('user/login'));
	exit;
}
template('account/welcome');

