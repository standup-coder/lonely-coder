<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
define('IN_API', true);
require './framework/bootstrap.inc.php';
load()->model('reply');
load()->app('common');
$hash = $_GPC['hash'];
if(!empty($hash)) {
	$id = pdo_fetchcolumn("SELECT id FROM " . tablename('account') . " WHERE hash = :hash", array(':hash' => $hash));
}
if(empty($hash)) {
	$id = intval($_GPC['id']);
}
$_W['account'] = account_fetch($id);
if(empty($_W['account'])) {
	exit('initial error hash');
}
if(empty($_W['account']['token'])) {
	exit('initial missing token');
}

$_W['acid'] = $_W['account']['acid'];
$_W['uniacid'] = $_W['account']['uniacid'];
$_W['modules'] = uni_modules();
if($_W['isajax'] && $_W['ispost'] && $_GPC['flag'] == 1) {
	$engine = new WeEngine();
	$engine->encrypt();
}
if($_W['isajax'] && $_W['ispost'] && $_GPC['flag'] == 2) {
	$engine = new WeEngine();
	$engine->decrypt();
}

$_W['isajax'] = false;
$engine = new WeEngine();
$engine->start();


class WeEngine {
	
	private $account = null;
	
	private $modules = array();
	
	public $keyword = array();

	
	public function __construct() {
		global $_W;
		$this->account = WeAccount::create($_W['acid']);
		$this->modules = array_keys($_W['modules']);
		$this->modules[] = 'cover';
		$this->modules[] = 'default';
		$this->modules = array_unique($this->modules);
	}

	
	public function encrypt() {
		global $_W;
		if(empty($this->account)) {
			exit('Miss Account.');
		}
		$timestamp = TIMESTAMP;
		$nonce = random(5);
		$token = $_W['account']['token'];
		$signkey = array($token, TIMESTAMP, $nonce);
		sort($signkey, SORT_STRING);
		$signString = implode($signkey);
		$signString = sha1($signString);

		$_GET['timestamp'] = $timestamp;
		$_GET['nonce'] = $nonce;
		$_GET['signature'] = $signString;
		$postStr = file_get_contents('php://input');
		if(!empty($_W['account']['encodingaeskey']) && strlen($_W['account']['encodingaeskey']) == 43 && !empty($_W['account']['key'])) {
			$data = $this->account->encryptMsg($postStr);
			$array = array('encrypt_type' => 'aes', 'timestamp' => $timestamp, 'nonce' => $nonce, 'signature' => $signString, 'msg_signature' => $data[0], 'msg' => $data[1]);
		} else {
			$data = array('', '');
			$array = array('encrypt_type' => '', 'timestamp' => $timestamp, 'nonce' => $nonce, 'signature' => $signString, 'msg_signature' => $data[0], 'msg' => $data[1]);
		}
		exit(json_encode($array));
	}

	
	public function decrypt() {
		global $_W;
		if(empty($this->account)) {
			exit('Miss Account.');
		}
		$postStr = file_get_contents('php://input');
		if(!empty($_W['account']['encodingaeskey']) && strlen($_W['account']['encodingaeskey']) == 43 && !empty($_W['account']['key'])) {
			$resp = $this->account->local_decryptMsg($postStr);
		} else {
			$resp = $postStr;
		}
		exit($resp);
	}

	
	public function start() {
		global $_W;
		if(empty($this->account)) {
			exit('Miss Account.');
		}
		if(!$this->account->checkSign()) {
			exit('Check Sign Fail.');
		}
		if(strtolower($_SERVER['REQUEST_METHOD']) == 'get') {
			$row = array();
			$row['isconnect'] = 1;
			pdo_update('account', $row, array('acid' => $_W['acid']));
			exit($_GET['echostr']);
		}
		if(strtolower($_SERVER['REQUEST_METHOD']) == 'post') {
			$postStr = file_get_contents('php://input');
						if(!empty($_GET['encrypt_type']) && $_GET['encrypt_type'] == 'aes') {
				$postStr = $this->account->decryptMsg($postStr);
			}
			$message = $this->account->parse($postStr);
			if(empty($message)) {
				WeUtility::logging('waring', 'Request Failed');
				exit('Request Failed');
			}
			$_W['openid'] = $message['from'];
			$this->booking($message);
			$sessionid = md5($message['from'] . $message['to'] . $_W['uniacid']);
			session_id($sessionid);
			WeSession::start($_W['uniacid'], $_W['openid']);

			WeUtility::logging('trace', $message);
			$pars = $this->analyze($message);
			$pars[] = array(
				'message' => $message,
				'module' => 'default',
				'rule' => '-1'
			);

			$hitParam = null;
			$hitKeyword = null;
			$response = null;
			foreach($pars as $par) {
				if(empty($par['module'])) {
					continue;
				}
				$par['message'] = $message;
				$response = $this->process($par);
				if($this->isValidResponse($response)) {
					$hitParam = $par;
					if(!empty($par['keyword'])) {
						$hitKeyword = $par['keyword'];
					}
					break;
				}
			}
			if($hitParam['module'] == 'default' && is_array($response) && is_array($response['params'])) {
				foreach($response['params'] as $par) {
					if(empty($par['module'])) {
						continue;
					}
					$response = $this->process($par);
					if($this->isValidResponse($response)) {
						$hitParam = $par;
						if(!empty($par['keyword'])) {
							$hitKeyword = $par['keyword'];
						}
						break;
					}
				}
			}

			WeUtility::logging('params', $hitParam);
			WeUtility::logging('response', $response);
			$resp = $this->account->response($response);
			$resp = $this->clip($resp, $hitParam);
						if(!empty($_GET['encrypt_type']) && $_GET['encrypt_type'] == 'aes') {
				$resp = $this->account->encryptMsg($resp);
				$resp = $this->account->xmlDetract($resp);
			}
			echo $resp;
			$this->enqueue($hitParam, $hitKeyword, $response);
			exit();
		}
		WeUtility::logging('waring', 'Request Failed');
		exit('Request Failed');
	}

	private function isValidResponse($response) {
		if(is_array($response)) {
			if($response['type'] == 'text' && !empty($response['content'])) {
				return true;
			}
			if($response['type'] == 'news' && !empty($response['items'])) {
				return true;
			}
			if(!in_array($response['type'], array('text', 'news', 'image'))) {
				return true;
			}
		}
		return false;
	}

	
	private function booking($message) {
		global $_W;
		$sql = 'SELECT follow, followtime, unfollowtime FROM ' . tablename('mc_mapping_fans') . ' WHERE `acid`=:acid AND `openid`=:openid';
		$pars = array();
		$pars[':acid'] = $_W['acid'];
		$pars[':openid'] = $message['from'];
		$fans = pdo_fetch($sql, $pars);
		
		if(!empty($fans)) {
			$rec = array();
			if (!empty($fans['follow'])) {
				if ($message['event'] == 'unsubscribe') {
					$rec['follow'] = 0;
					$rec['followtime'] = 0;
					$rec['unfollowtime'] = $message['time'];
				}
			} else {
				if ($message['event'] != 'unsubscribe') {
					$rec['follow'] = 1;
					$rec['followtime'] = $message['time'];
					$rec['unfollowtime'] = 0;
				}
			}
			if(!empty($rec)){
				pdo_update('mc_mapping_fans', $rec, array(
					'acid' => $_W['acid'],
					'openid' => $message['from']
				));
			}
		} else {
			$rec = array();
			$rec['acid'] = $_W['acid'];
			$rec['uniacid'] = $_W['uniacid'];
			$rec['uid'] = 0;
			$rec['openid'] = $message['from'];
			$rec['salt'] = random(8);
			if ($message['event'] == 'unsubscribe') {
				$rec['follow'] = 0;
				$rec['followtime'] = 0;
				$rec['unfollowtime'] = $message['time'];
			} else {
				$rec['follow'] = 1;
				$rec['followtime'] = $message['time'];
				$rec['unfollowtime'] = 0;
			}
			pdo_insert('mc_mapping_fans', $rec);
		}
	}

	
	private function clip($resp, $par) {
		$mapping = array(
			'[from]' => $par['message']['from'],
			'[to]' => $par['message']['to'],
			'[rule]' => $par['rule']
		);

		return str_replace(array_keys($mapping), array_values($mapping), $resp);
	}

	private function enqueue($par, $keyword, $response) {
		global $_W;
		$row = array();
		$row['uniacid'] = $_W['uniacid'];
		$row['acid'] = $_W['acid'];
		$row['dateline'] = $par['message']['time'];
		$row['message'] = iserializer($par['message']);
		$row['keyword'] = iserializer($keyword);
		unset($par['message']);
		unset($par['keyword']);
		$row['params'] = iserializer($par);
		$row['response'] = iserializer($response);
		$row['module'] = $par['module'];
		pdo_insert('core_queue', $row);
	}

	
	private function analyze(&$message) {
		$params = array();
		if(in_array($message['type'], array('event', 'qr'))) {
			$params = call_user_func(array($this, 'analyze' . $message['type']), $message);
			if(!empty($params)) {
				return (array)$params;
			}
		}
		if(!empty($_SESSION['__contextmodule']) && in_array($_SESSION['__contextmodule'], $this->modules)) {
			if($_SESSION['__contextexpire'] > TIMESTAMP) {
				if($_SESSION['__contextpriority'] < 255 && $message['type'] == 'text') {
					$params += $this->analyzeText($message, intval($_SESSION['__contextpriority']));
				}
				$params[] = array(
					'message' => $message,
					'module' => $_SESSION['__contextmodule'],
					'rule' => $_SESSION['__contextrule'],
					'priority' => $_SESSION['__contextpriority'],
					'context' => true
				);
				return $params;
			} else {
				unset($_SESSION);
				session_destroy();
			}
		}

		if(method_exists($this, 'analyze' . $message['type'])) {
			$temp = call_user_func(array($this, 'analyze' . $message['type']), $message);
			if(!empty($temp) && is_array($temp)){
				$params += $temp;
			}
		} else {
			$params += $this->handler($message['type']);
		}

		return $params;
	}
	
	private function analyzeSubscribe(&$message) {
		global $_W;
		$params = array();
		$message['type'] = 'text';
		$message['redirection'] = true;
		if(!empty($message['scene'])) {
			$message['source'] = 'qr';
			$sceneid = floatval($message['scene']);
			$qr = pdo_fetch("SELECT `id`, `keyword` FROM " . tablename('qrcode') . " WHERE `qrcid` = '{$sceneid}' AND `uniacid` = '{$_W['uniacid']}'");
			if(!empty($qr)) {
				$message['content'] = $qr['keyword'];
				$params += $this->analyzeText($message);
			}
		}
		$message['source'] = 'subscribe';
		$setting = uni_setting($_W['uniacid'], array('welcome'));
		if(!empty($setting['welcome'])) {
			$message['content'] = $setting['welcome'];
			$params += $this->analyzeText($message);
		}

		return $params;
	}

	private function analyzeQR(&$message) {
		global $_W;
		$params = array();
		$message['type'] = 'text';
		$message['redirection'] = true;
		if(!empty($message['scene'])) {
			$message['source'] = 'qr';
			$sceneid = floatval($message['scene']);
			$qr = pdo_fetch("SELECT `id`, `keyword` FROM " . tablename('qrcode') . " WHERE `qrcid` = '{$sceneid}' AND `uniacid` = '{$_W['uniacid']}'");
			if(!empty($qr)) {
				$message['content'] = $qr['keyword'];
				$params += $this->analyzeText($message);
			}
		}

		return $params;
	}

	public function analyzeText(&$message, $order = -1) {
		$pars = array();
		$input = $message['content'];
		if(!isset($input)) {
			return $pars;
		}
		global $_W;
		$order = intval($order);
		$condition = "`status`=1 AND (`uniacid`='{$_W['uniacid']}' OR `uniacid`=0) AND `displayorder`>{$order}";
		$condition .= " AND (((`type` = '1' OR `type` = '2') AND `content` = :c1) OR (`type` = '4') OR (`type` = '3' AND :c2 REGEXP `content`) OR (`type` = '2' AND INSTR(:c3, `content`) > 0))";
		$params = array();
		$params[':c1'] = $input;
		$params[':c2'] = $input;
		$params[':c3'] = $input;
		$keywords = reply_keywords_search($condition, $params);
		if(empty($keywords)) {
			return $pars;
		}
		foreach($keywords as $kwd) {
			$params = array(
				'message' => $message,
				'module' => $kwd['module'],
				'rule' => $kwd['rid'],
				'priority' => $kwd['displayorder'],
				'keyword' => $kwd
			);
			$pars[] = $params;
		}

		return $pars;
	}
	
	private function analyzeEvent(&$message) {
		if (strtolower($message['event']) == 'subscribe') {
			return $this->analyzeSubscribe($message);
		}
		if (strtolower($message['event']) == 'click') {
			$message['content'] = strval($message['eventkey']);
			return $this->analyzeClick($message);
		}
		if (in_array($message['event'], array('pic_photo_or_album', 'pic_weixin', 'pic_sysphoto'))) {
			if (!empty($message['sendpicsinfo']['count'])) {
				foreach ($message['sendpicsinfo']['piclist'] as $item) {
					pdo_insert('menu_event', array(
						'uniacid' => $GLOBALS['_W']['uniacid'],
						'keyword' => $message['eventkey'],
						'type' => $message['event'],
						'picmd5' => $item,
					));
				}
			}
			return true;
		}
		if (!empty($message['eventkey'])) {
			$message['content'] = strval($message['eventkey']);
			$message['type'] = 'text';
			$message['redirection'] = true;
			$message['source'] = $message['event'];
			return $this->analyzeText($message);
		}
	}
	
	private function analyzeClick(&$message) {
		if(!empty($message['content'])) {
			$message['type'] = 'text';
			$message['redirection'] = true;
			$message['source'] = 'click';
	
			return $this->analyzeText($message);
		}
	
		return array();
	}
	
	private function analyzeImage(&$message) {
		load()->func('communication');
		if (!empty($message['picurl'])) {
			$response = ihttp_get($message['picurl']);
			if (!empty($response)) {
				$md5 = md5($response['content']);
				$event = pdo_fetch("SELECT keyword, type FROM ".tablename('menu_event')." WHERE picmd5 = '$md5'");
				if (!empty($event['keyword'])) {
					pdo_delete('menu_event', array('picmd5' => $md5));
					$message['content'] = $event['keyword'];
					$message['type'] = 'text';
					$message['redirection'] = true;
					$message['source'] = $event['type'];
					return $this->analyzeText($message);
				}
			}
			return $this->handler('image');
		}
	}

	
	private function handler($type) {
		if(empty($type)) {
			return array();
		}
		global $_W;
		$params = array();

		$setting = uni_setting($_W['uniacid'], array('default_message'));
		$df = $setting['default_message'];
		if(is_array($df) && isset($df[$type]) && in_array($df[$type], $this->modules)) {
			$params[] = array(
				'module' => $df[$type],
				'rule' => '-1',
			);
		}

		return $params;
	}

	
	private function process($param) {
		global $_W;
		if(empty($param['module']) || !in_array($param['module'], $this->modules)) {
			return false;
		}

		$processor = WeUtility::createModuleProcessor($param['module']);
		$processor->message = $param['message'];
		$processor->rule = $param['rule'];
		$processor->priority = intval($param['priority']);
		$processor->inContext = $param['context'] === true;
		$response = $processor->respond();
		if(empty($response)) {
			return false;
		}

		return $response;
	}
}


class WeSession {
	
	public static $uniacid;
	
	public static $openid;
	
	public static $expire;

	
	public static function start($uniacid, $openid, $expire = 3600) {
		WeSession::$uniacid = $uniacid;
		WeSession::$openid = $openid;
		WeSession::$expire = $expire;
		$sess = new WeSession();
		session_set_save_handler(
			array(&$sess, 'open'),
			array(&$sess, 'close'),
			array(&$sess, 'read'),
			array(&$sess, 'write'),
			array(&$sess, 'destroy'),
			array(&$sess, 'gc')
		);
		session_start();
	}

	public function open() {
		return true;
	}

	public function close() {
		return true;
	}

	
	public function read($sessionid) {
		$sql = 'SELECT * FROM ' . tablename('core_sessions') . ' WHERE `sid`=:sessid AND `expiretime`>:time';
		$params = array();
		$params[':sessid'] = $sessionid;
		$params[':time'] = TIMESTAMP;
		$row = pdo_fetch($sql, $params);
		if(is_array($row) && !empty($row['data'])) {
			return $row['data'];
		}
		return false;
	}

	
	public function write($sessionid, $data) {
		$row = array();
		$row['sid'] = $sessionid;
		$row['uniacid'] = WeSession::$uniacid;
		$row['openid'] = WeSession::$openid;
		$row['data'] = $data;
		$row['expiretime'] = TIMESTAMP + WeSession::$expire;

		return pdo_insert('core_sessions', $row, true) == 1;
	}

	
	public function destroy($sessionid) {
		$row = array();
		$row['sid'] = $sessionid;

		return pdo_delete('core_sessions', $row) == 1;
	}

	
	public function gc($expire) {
		$sql = 'DELETE FROM ' . tablename('core_sessions') . ' WHERE `expiretime`<:expire';

		return pdo_query($sql, array(':expire' => TIMESTAMP)) == 1;
	}
}