<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');

class NewsModule extends WeModule {
	public $tablename = 'news_reply';
	public $replies = array();

	public function fieldsFormDisplay($rid = 0) {
		global $_W;
		load()->func('tpl');
		$replies = pdo_fetchall("SELECT * FROM ".tablename($this->tablename)." WHERE rid = :rid ORDER BY `displayorder` DESC", array(':rid' => $rid));
		foreach($replies as &$reply) {
			if(!empty($reply['thumb'])) {
				$reply['src'] = tomedia($reply['thumb']);
			}
		}
		include $this->template('display');
	}
	
	public function fieldsFormValidate($rid = 0) {
		global $_GPC;
		if(empty($_GPC['titles'])) {
			return '必须填写有效的回复内容.';
		}
		foreach($_GPC['titles'] as $k => $v) {
			$row = array();
			if(empty($v)) {
				continue;
			}
			$row['title'] = $v;
			$row['displayorder'] = $_GPC['displayorder'][$k];
			$row['thumb'] = $_GPC['thumbs'][$k];
			$row['description'] = $_GPC['descriptions'][$k];
			$row['content'] = $_GPC['contents'][$k];
			$row['url'] = $_GPC['urls'][$k];
			$row['incontent'] = intval($_GPC['incontent'][$k]);
			$this->replies[] = $row;
		}
		if(empty($this->replies)) {
			return '必须填写有效的回复内容.';
		}
		foreach($this->replies as &$r) {
			if(trim($r['title']) == '' || trim($r['thumb']) == '') {
				return '必须填写有效的回复内容.';
			}
			$r['content'] = htmlspecialchars_decode($r['content']);
		}
		return '';
	}
	
	public function fieldsFormSubmit($rid = 0) {
		$sql = 'DELETE FROM '. tablename($this->tablename) . ' WHERE `rid`=:rid';
		$pars = array();
		$pars[':rid'] = $rid;
		pdo_query($sql, $pars);
		foreach($this->replies as $reply) {
			$reply['rid'] = $rid;
			pdo_insert($this->tablename, $reply);
		}
		return true;	
	}
	
	public function ruleDeleted($rid = 0) {
		pdo_delete($this->tablename, array('rid' => $rid));
		return true;
	}
}