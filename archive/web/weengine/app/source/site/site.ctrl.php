<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
$do = in_array($do, array('list', 'detail', 'handsel')) ? $do : 'list';
load()->model('site');
load()->model('mc');

if($do == 'list') {
	$cid = intval($_GPC['cid']);
	$category = pdo_fetch("SELECT * FROM ".tablename('site_category')." WHERE id = '{$cid}' AND uniacid = '{$_W['uniacid']}'");
	if (empty($category)) {
		message('分类不存在或是已经被删除！');
	}
	if (!empty($category['linkurl'])) {
		header('Location: '.$category['linkurl']);
		exit;
	}
	$title = $category['name'];
	$category['template'] = pdo_fetchcolumn('SELECT b.name FROM ' . tablename('site_styles') . ' AS a LEFT JOIN ' . tablename('site_templates') . ' AS b ON a.templateid = b.id WHERE a.id = :id', array(':id' => $category['styleid']));
	if(!empty($category['template'])) {
		$styles_vars = pdo_fetchall('SELECT * FROM ' . tablename('site_styles_vars') . ' WHERE styleid = :styleid', array(':styleid' => $category['styleid']));
		if(!empty($styles_vars)) {
			foreach($styles_vars as $row) {
				if (strexists($row['variable'], 'img')) {
					$row['content'] = tomedia($row['content']);
				}
				$_W['styles'][$row['variable']] = $row['content'];
			}
		}
	}
	if (empty($category['ishomepage'])) {
				if(!empty($category['template'])) {
			$_W['template'] = $category['template'];
		}
		if (!empty($category['templatefile'])) {
			$category['templatefile'] = iunserializer($category['templatefile']);
			template('site/' . $category['templatefile']['list']);
			exit;
		} else {
			template('site/list');
			exit;
		}
	} else {
		if(!empty($category['template'])) {
			$_W['template'] = $category['template'];
		}
		$navs = pdo_fetchall("SELECT * FROM ".tablename('site_category')." WHERE uniacid = '{$_W['uniacid']}' AND parentid = '$cid' ORDER BY displayorder ASC");
		if (!empty($navs)) {
			foreach ($navs as &$row) {
				$row['url'] = url('site/site/list', array('cid' => $row['id']));
				if (!empty($row['icontype']) && $row['icontype'] == 1) {
					$row['css'] = iunserializer($row['css']);
					$row['icon'] = '';
					$row['css']['icon']['style'] = "color:{$row['css']['icon']['color']};font-size:{$row['css']['icon']['font-size']}px;";
					$row['css']['name'] = "color:{$row['css']['name']['color']};";
				}
				if (!empty($row['icontype']) && $row['icontype'] == 2) {
					$row['css'] = '';
				}
			}
		}
		if (!empty($category['templatefile'])) {
			$category['templatefile'] = iunserializer($category['templatefile']);
			template('site/' . $category['templatefile']['list']);
			exit;
		} else {
			template('home/home');
			exit;
		}
	}	
} elseif($do == 'detail') {
	$id = intval($_GPC['id']);
	$sql = "SELECT * FROM " . tablename('site_article') . " WHERE `id`=:id AND uniacid = :uniacid";
	$detail = pdo_fetch($sql, array(':id'=>$id, ':uniacid' => $_W['uniacid']));
	if (!empty($detail['linkurl'])) {
		if(!strexists($detail['linkurl'], 'http://') && !strexists($detail['linkurl'], 'https://')) {
			$detail['linkurl'] = $_W['siteroot'] . 'app/' . $detail['linkurl'];
		}
		header('Location: '. $detail['linkurl']);
		exit;
	}
	$detail = istripslashes($detail);
	if(!empty($detail['thumb'])) {
		$detail['thumb'] = tomedia($detail['thumb']);
	} else {
		$detail['thumb'] = '';
	}
	$title = $detail['title'];
		if(!empty($detail['template'])) {
		$_W['template'] = $detail['template'];
	}
	template('site/detail');
} elseif($do == 'handsel') {
		if($_W['ispost']) {
		$id = intval($_GPC['id']);
		$article = pdo_fetch('SELECT id, credit FROM ' . tablename('site_article') . ' WHERE uniacid = :uniacid AND id = :id', array(':uniacid' => $_W['uniacid'], ':id' => $id));
		$credit = iunserializer($article['credit']) ? iunserializer($article['credit']) : array();
		if(!empty($article) && $credit['status'] == 1) {
			if($_GPC['action'] == 'share') {
				$touid = $_W['member']['uid'];
				$formuid = -1;
				$handsel = array('module' => 'article', 'sign' => md5(iserializer(array('id' => $id))), 'action' => 'share', 'credit_value' => $credit['share']);
			} elseif($_GPC['action'] == 'click' && !empty($_GPC['u'])) {
				$touid = intval($_GPC['u']);
				$formuid = $_W['member']['uid'];
				$handsel = array('module' => 'article', 'sign' => md5(iserializer(array('id' => $id))), 'action' => 'click', 'credit_value' => $credit['click']);
			}
			$total = pdo_fetchcolumn('SELECT SUM(credit_value) FROM ' . tablename('mc_handsel') . ' WHERE uniacid = :uniacid AND module = :module AND sign = :sign', array(':uniacid' => $_W['uniacid'], ':module' => 'article', ':sign' => $handsel['sign']));
			if($total >= $credit['limit']) {
				exit(json_encode(array(-1, '赠送积分已达到上限')));
			}
			$status = mc_handsel($touid, $formuid, $handsel, $_W['uniacid']);
			if(is_error($status)) {
				exit(json_encode($status));
			} else {
				exit('success');
			}
		} else {
			exit(json_encode(array(-1, '文章没有设置赠送积分')));
		} 
	} else {
		exit(json_encode(array(-1, '非法操作')));
	}
}
