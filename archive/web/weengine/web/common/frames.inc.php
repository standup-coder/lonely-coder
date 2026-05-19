<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');

$ms = array();
$ms['platform'][] =  array(
	'title' => '基本功能',
	'items' => array(
		array(
			'title' => '文字回复',
			'url' => url('platform/reply', array('m' => 'basic')),
			'append' => array(
				'title' => '<i class="fa fa-plus"></i>', 
				'url' => url('platform/reply/post', array('m' => 'basic'))
			)
		),
		array(
			'title' => '图文回复',
			'url' => url('platform/reply', array('m' => 'news')),
			'append' => array(
				'title' => '<i class="fa fa-plus"></i>', 
				'url' => url('platform/reply/post', array('m' => 'news')),
			),
		),
		array(
			'title' => '音乐回复',
			'url' => url('platform/reply', array('m' => 'music')),
			'append' => array(
				'title' => '<i class="fa fa-plus"></i>', 
				'url' => url('platform/reply/post', array('m' => 'music'))
			),
		),
		array(
			'title' => '自定义接口回复',
			'url' => url('platform/reply', array('m' => 'userapi')),
			'append' => array(
				'title' => '<i class="fa fa-plus"></i>', 
				'url' => url('platform/reply/post', array('m' => 'userapi')),
			),
		),
	)
);
$ms['platform'][] =  array(
	'title' => '高级功能',
	'items' => array(
		array('title' => '常用服务接入', 	'url' => url('platform/service/switch')),
		array('title' => '自定义菜单', 	'url' => url('platform/menu')),
		array('title' => '特殊回复', 		'url' => url('platform/special')),
		array('title' => '二维码管理', 	'url' => url('platform/qr')),
			)
);
$ms['platform'][] =  array(
	'title' => '数据统计',
	'items' => array(
		array('title' => '聊天记录', 			'url' => url('platform/stat/history')),
		array('title' => '回复规则使用情况', 	'url' => url('platform/stat/rule')),
		array('title' => '关键字命中情况', 	'url' => url('platform/stat/keyword')),
		array('title' => '参数', 			'url' => url('platform/stat/setting'))
	)
);
$ms['site'][] =  array(
	'title' => '风格管理',
	'items' => array(
		array('title' => '风格管理', 		'url' => url('site/style/styles')),
		array('title' => '模板管理', 		'url' => url('site/style/template')),
		array('title' => '模块扩展模板说明', 'url' => url('site/style/module')),
	)
);
$ms['site'][] =  array(
		'title' => '微站管理',
		'items' => array(
			array('title' => '微站管理',		'url' => url('site/multi/display')),
			array('title' => '添加微站',		'url' => url('site/multi/post')),
		)
);
$ms['site'][] =  array(
	'title' => '导航及菜单',
	'items' => array(
		array('title' => '微站首页导航图标', 	'url' => url('site/nav/home')),
		array('title' => '个人中心功能条目', 	'url' => url('site/nav/profile')),
		array('title' => '快捷菜单',			'url' => url('site/nav/shortcut')),
	)
);
$ms['site'][] =  array(
	'title' => '功能组件',
	'items' => array(
		array('title' => '幻灯片设置', 		'url' => url('site/slide')),
		array('title' => '分类设置', 			'url' => url('site/category')),
		array('title' => '文章管理', 			'url' => url('site/article')),
	)
);

$ms['mc'][] = array(
	'title' => '会员中心',
	'items' => array(
		array('title' => '粉丝', 			'url' => url('mc/fans')),
		array('title' => '会员', 			'url' => url('mc/member')),
		array('title' => '会员组', 			'url' => url('mc/group')),
		array('title' => '会员积分管理', 		'url' => url('mc/creditmanage')),
		array('title' => '积分设置', 			'url' => url('mc/credit')),
		array('title' => '会员中心选项', 		'url' => url('mc/passport')),
		array('title' => '会员中心访问入口', 	'url' => url('platform/cover/mc')),
		array('title' => '操作店员管理', 		'url' => url('activity/offline'))
	)
);
$ms['mc'][] = array(
	'title' => '会员卡管理',
	'items' => array(
			array('title' => '会员卡管理', 	'url' => url('mc/card')),
			array('title' => '商家设置',	'url' =>url('mc/business')),
			array('title' => '会员卡访问入口', 	'url' => url('platform/cover/card'))
	)
);
$ms['mc'][] = array(
	'title' => '积分兑换',
	'items' => array(
		array('title' => '折扣券', 		'url' => url('activity/coupon')),
		array('title' => '代金券', 		'url' => url('activity/token')),
		array('title' => '真实物品',	'url' => url('activity/goods')),
			)
);
$ms['mc'][] = array(
	'title' => '通知中心',
	'items' => array(
		array('title' => '群发消息&通知', 	'url' => url('mc/broadcast')),
		array('title' => '通知参数', 		'url' => url('profile/notify')),
	)
);

$ms['setting'][] = array(
	'title' => '公众号选项',
	'items' => array(
		array('title' => '支付参数', 		'url' => url('profile/payment')),
	)
);
$ms['setting'][] = array(
	'title' => '管理人员',
	'items' => array(
		array('title' => '操作人员列表', 	'url' => url('profile/worker')),
	)
);
$ms['setting'][] = array(
	'title' => '其他功能选项',
	'items' => array(
			)
);

$ms['ext'][] = array(
	'title' => '管理',
	'items' => array(
		array('title' => '扩展功能管理', 'url' => url('profile/module'))
	)
);

return $ms;
