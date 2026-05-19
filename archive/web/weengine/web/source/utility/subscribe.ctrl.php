<?php 
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');

if(!empty($_W['uniacid'])) {
	$sql = 'SELECT * FROM ' . tablename('core_queue') . ' WHERE `uniacid`=:uniacid ORDER BY `dateline` LIMIT 100';
	$pars = array();
	$pars[':uniacid'] = $_W['uniacid'];
	$messages = pdo_fetchall($sql, $pars);
	
	if (!empty($messages)) {
		
		$qids = '';
		foreach($messages as &$message) {
			$message['message'] = iunserializer($message['message']);
			$message['params'] = iunserializer($message['params']);
			$message['response'] = iunserializer($message['response']);
			$message['keyword'] = iunserializer($message['keyword']);
		
			$qids .= $message['qid'] . ',';
		}
		
		$qids = trim($qids, ',');
		$sql = 'DELETE FROM ' . tablename('core_queue') . " WHERE `qid` IN ({$qids})";
		pdo_query($sql);
		
		load()->model('module');
		$modules = uni_modules();
		$core = array();
		$core['name'] = 'core';
		$core['subscribes'] = array('core');
		array_unshift($modules, $core);
		
		foreach($messages as $msg) {
			foreach($modules as $m) {
				if(!empty($m['subscribes'])) {
					if($m['name'] == 'core' || in_array($msg['message']['type'], $m['subscribes'])) {
						$obj = WeUtility::createModuleReceiver($m['name']);
						$obj->message = $msg['message'];
						$obj->params = $msg['params'];
						$obj->response = $msg['response'];
						$obj->keyword = $msg['keyword'];
						$obj->module = $m;
						if(method_exists($obj, 'receive')) {
							$obj->receive();
						}
					}
				}
			}
		}
	}
}