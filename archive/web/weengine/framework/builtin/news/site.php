<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');

class NewsModuleSite extends WeModuleSite {

	public function doMobileDetail() {
		global $_W, $_GPC;
		$id = intval($_GPC['id']);
		$sql = "SELECT * FROM " . tablename('news_reply') . " WHERE `id`=:id";
		$row = pdo_fetch($sql, array(':id'=>$id));
		if (!empty($row['url'])) {
			header("Location: ".$row['url']);
		}
		$row = istripslashes($row);
		$row['thumb'] = $_W['attachurl'] . trim($row['thumb'], '/');
				if ($row['incontent'] == 0) {
			$row['thumb'] = '';
		}
		$title = $row['title'];
		include $this->template('detail');
	}
}