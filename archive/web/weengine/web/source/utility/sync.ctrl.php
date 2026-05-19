<?php 
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');

if(!empty($_W['uniacid'])) {
	load()->model('account');
	load()->model('mc');
	$setting = uni_setting($_W['uniacid'], 'sync');
	$sync = is_array($setting['sync']) ? $setting['sync'] : array('switch' => 0, 'acid' => '');
	if($sync['switch'] == 1 && !empty($sync['acid'])) {
		$data = pdo_fetchall('SELECT fanid, openid, acid, uid, uniacid FROM ' . tablename('mc_mapping_fans') . " WHERE uniacid = :uniacid AND tag = '' ORDER BY fanid DESC LIMIT 10", array(':uniacid' => $_W['uniacid']));
		if(!empty($data)) {
			$acc = WeAccount::create($sync['acid']);
			foreach($data as $row) {
				$fan = $acc->fansQueryInfo($row['openid'], true);
 				if(!is_error($fan)) {
					$record = array();
					$record['followtime'] = $fan['subscribe_time'];
					$record['tag'] = iserializer($fan);
					pdo_update('mc_mapping_fans', $record, array('fanid' => $row['fanid']));
					if(!empty($row['uid'])) {
						$user = mc_fetch($row['uid'], array('nickname', 'gender', 'residecity', 'resideprovince', 'nationality', 'avatar'));
						$rec = array();
						if(empty($user['nickname']) && !empty($fan['nickname'])) {
							$rec['nickname'] = $fan['nickname'];
						}
						if(empty($user['gender']) && !empty($fan['sex'])) {
							$rec['gender'] = $fan['sex'];
						}
						if(empty($user['residecity']) && !empty($fan['city'])) {
							$rec['residecity'] = $fan['city'] . '市';
						}
						if(empty($user['resideprovince']) && !empty($fan['province'])) {
							$rec['resideprovince'] = $fan['province'] . '省';
						}
						if(empty($user['nationality']) && !empty($fan['country'])) {
							$rec['nationality'] = $fan['country'];
						}
						if(empty($user['avatar']) && !empty($fan['headimgurl'])) {
							$rec['avatar'] = rtrim($fan['headimgurl'], '0') . 132;
						}
						if(!empty($rec)) {
							pdo_update('mc_members', $rec, array('uid' => $row['uid']));
						}
					}
				} 
			}
		}
	}
}