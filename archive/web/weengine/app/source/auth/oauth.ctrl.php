<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');

$setting = uni_setting($_W['uniacid'], array('oauth'));
$oauth = $setting['oauth'];
if(!empty($oauth['status']) && !empty($oauth['account'])) {
	$account = account_fetch($oauth['account']);
	$code = $_GPC['code'];

	if(!empty($code)) {
		$url = "https://api.weixin.qq.com/sns/oauth2/access_token?appid={$account['key']}&secret={$account['secret']}&code={$code}&grant_type=authorization_code";
		$ret = ihttp_get($url);
		if(!is_error($ret)) {
			$auth = @json_decode($ret['content'], true);
			if(is_array($auth) && !empty($auth['openid'])) {
				$sql = 'SELECT `fanid`,`salt`,`uid` FROM ' . tablename('mc_mapping_fans') . ' WHERE `uniacid`=:uniacid AND `acid`=:acid AND `openid`=:openid';
				$pars = array();
				$pars[':uniacid'] = $_W['uniacid'];
				$pars[':acid'] = $account['acid'];
				$pars[':openid'] = $auth['openid'];
				$fan = pdo_fetch($sql, $pars);
				if(empty($fan)) {
					$fan = array();
					$fan['acid'] = $_W['uniacid'];
					$fan['uniacid'] = $account['acid'];
					$fan['uid'] = 0;
					$fan['openid'] = $auth['openid'];
					$fan['salt'] = random(8);
					$fan['follow'] = 0;
					$fan['followtime'] = 0;
					pdo_insert('mc_mapping_fans', $fan);
				}
				$_SESSION['openid'] = $auth['openid'];
				
				$state = $_SESSION[$_GPC['state']];
				$forward = base64_decode($state);
				header('location: ' . $_W['siteroot'] . 'app/index.php?' . $forward . '&wxref=mp.weixin.qq.com#wechat_redirect');
				exit;
			}
		}
		message('微信授权失败错误信息为: ' . $ret['message']);
	}
}
exit('访问错误');
