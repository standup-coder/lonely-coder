<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
$do = !empty($do) ? $do : 'display';
$do = in_array($do, array('display', 'post', 'delete', 'fetch', 'templatefiles')) ? $do : 'display';
$setting = uni_setting($_W['uniacid'], 'default_site');
$default_site = intval($setting['default_site']);

if ($do == 'display') {
	if (!empty($_GPC['displayorder'])) {
		foreach ($_GPC['displayorder'] as $id => $displayorder) {
			pdo_update('site_category', array('displayorder' => $displayorder), array('id' => $id));
		}
		message('分类排序更新成功！', 'refresh', 'success');
	}
	$children = array();
	$category = pdo_fetchall("SELECT * FROM ".tablename('site_category')." WHERE uniacid = '{$_W['uniacid']}' ORDER BY parentid ASC, displayorder DESC, id ASC ");
	foreach ($category as $index => $row) {
		if (!empty($row['parentid'])){
			$children[$row['parentid']][] = $row;
			unset($category[$index]);
		}
	}
	template('site/category');
} elseif ($do == 'post') {
	load()->func('tpl');
	$parentid = intval($_GPC['parentid']);
	$id = intval($_GPC['id']);
		$styles = pdo_fetchall("SELECT a.*, b.name AS tname, b.title FROM ".tablename('site_styles').' AS a LEFT JOIN ' . tablename('site_templates') . ' AS b ON a.templateid = b.id WHERE a.uniacid = :uniacid', array(':uniacid' => $_W['uniacid']));
	if(!empty($id)) {
		$category = pdo_fetch("SELECT * FROM ".tablename('site_category')." WHERE id = '$id' AND uniacid = {$_W['uniacid']}");
		if(empty($category)) {
			message('分类不存在或已删除', '', 'error');
		}
		if (!empty($category['css'])) {
			$category['css'] = iunserializer($category['css']);
		} else {
			$category['css'] = array();
		}
		$category['template'] = pdo_fetchcolumn('SELECT b.name FROM ' . tablename('site_styles') . ' AS a LEFT JOIN ' . tablename('site_templates') . ' AS b ON a.templateid = b.id WHERE a.id = :id', array(':id' => $category['styleid']));
		if (!empty($category['template'])) {
			$files = array();
			if ($category['ishomepage']) {
				$path = IA_ROOT . '/app/themes/' . $category['template'];
				$strexists = 'index';
			} else {
				$path = IA_ROOT . '/app/themes/' . $category['template'] . '/site';
				$strexists = '.html';
			}
			if (is_dir($path)) {
				if ($handle = opendir($path)) {
					while (false !== ($filepath = readdir($handle))) {
						if ($filepath != '.' && $filepath != '..' && strexists($filepath, $strexists)) {
							$files[] = $filepath;
						}
					}
				}
			}
		}
	} else {
		$category = array(
			'displayorder' => 0,
			'css' => array(),
		);
	}
	if (!empty($parentid)) {
		$parent = pdo_fetch("SELECT id, name FROM ".tablename('site_category')." WHERE id = '$parentid'");
		if (empty($parent)) {
			message('抱歉，上级分类不存在或是已经被删除！', url('site/category/display'), 'error');
		}
	}

	if (checksubmit('submit')) {
		if (empty($_GPC['cname'])) {
			message('抱歉，请输入分类名称！');
		}
		$data = array(
			'uniacid' => $_W['uniacid'],
			'name' => $_GPC['cname'],
			'displayorder' => intval($_GPC['displayorder']),
			'parentid' => intval($parentid),
			'description' => $_GPC['description'],
			'styleid' => intval($_GPC['styleid']),
			'templatefile' => iserializer(array('list' => 'list', 'detail' => 'detail')),
			'linkurl' => $_GPC['linkurl'],
			'ishomepage' => intval($_GPC['ishomepage']),
		);
		
		$data['icontype'] = intval($_GPC['icontype']);
		if($data['icontype'] == 1) {
			$data['css'] = serialize(array(
				'icon' => array(
					'font-size' => $_GPC['icon']['size'],
					'color' => $_GPC['icon']['color'],
					'width' => $_GPC['icon']['size'],
					'icon' => $_GPC['icon']['icon'],
				),
			));
		} else {
			$data['icon'] = $_GPC['iconfile'];
		}
		
		$isnav = intval($_GPC['isnav']);
		if($isnav) {
			$nav = array(
				'uniacid' => $_W['uniacid'],
				'name' => $_GPC['cname'],
				'description' => $_GPC['description'],
				'url' => "./index.php?c=site&a=site&cid={$category['id']}&i={$_W['uniacid']}",
				'status' => 1,
				'position' => 1,
				'multiid' => $default_site,
			);
			if ($data['icontype'] == 1) {
				$nav['icon'] = '';
				$nav['css'] = serialize(array(
					'icon' => array(
						'font-size' => $_GPC['icon']['size'],
						'color' => $_GPC['icon']['color'],
						'width' => $_GPC['icon']['size'],
						'icon' => $_GPC['icon']['icon'],
					),
					'name' => array(
						'color' => $_GPC['icon']['color'],
					),
				));
			} else {
				$nav['css'] = '';
				$nav['icon'] = $_GPC['iconfile'];
			}
			if($category['nid']) {
				$nav_exist = pdo_fetch('SELECT id FROM ' . tablename('site_nav') . ' WHERE id = :id AND uniacid = :uniacid', array(':id' => $category['nid'], ':uniacid' => $_W['uniacid']));
			} else {
				$nav_exist = '';
			}
			if(!empty($nav_exist)) {
				pdo_update('site_nav', $nav, array('id' => $category['nid'], 'uniacid' => $_W['uniacid']));
			} else {
				pdo_insert('site_nav', $nav);
				$nid = pdo_insertid();
				$data['nid'] = $nid;
			}
		} else {
			if($category['nid']) {
				$data['nid'] = 0;
				pdo_delete('site_nav', array('id' => $category['nid'], 'uniacid' => $_W['uniacid']));
			}
		}
		if (!empty($id)) {
			unset($data['parentid']);
			pdo_update('site_category', $data, array('id' => $id));
		} else {
			pdo_insert('site_category', $data);
			$id = pdo_insertid();
			$nav_url['url'] = "./index.php?c=site&a=site&cid={$id}&i={$_W['uniacid']}";
			pdo_update('site_nav', $nav_url, array('id' => $data['nid'], 'uniacid' => $_W['uniacid']));
		}
		message('更新分类成功！', url('site/category'), 'success');
	}
	template('site/category');
} elseif ($do == 'fetch') {
	$category = pdo_fetchall("SELECT id, name FROM ".tablename('site_category')." WHERE parentid = '".intval($_GPC['parentid'])."' ORDER BY id ASC, displayorder ASC, id ASC ");
	message($category, '', 'ajax');
} elseif ($do == 'delete') {
	load()->func('file');
	$id = intval($_GPC['id']);
	$category = pdo_fetch("SELECT id, parentid, nid FROM ".tablename('site_category')." WHERE id = '$id'");
	if (empty($category)) {
		message('抱歉，分类不存在或是已经被删除！', url('site/category/display'), 'error');
	}
	$navs = pdo_fetchall("SELECT icon, id FROM ".tablename('site_nav')." WHERE id IN (SELECT nid FROM ".tablename('site_category')." WHERE id = {$id} OR parentid = '$id')", array(), 'id');
	if (!empty($navs)) {
		foreach ($navs as $row) {
			file_delete($row['icon']);
		}
		pdo_query("DELETE FROM ".tablename('site_nav')." WHERE id IN (".implode(',', array_keys($navs)).")");
	}
	pdo_delete('site_category', array('id' => $id, 'parentid' => $id), 'OR');
	message('分类删除成功！', url('site/category'), 'success');
}