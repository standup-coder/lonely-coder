<?php
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
defined('IN_IA') or exit('Access Denied');
error_reporting(0);

$do = !empty($_GPC['do']) ? $_GPC['do'] : exit('Access Denied');

$type = $_GPC['type'];
$types = array('image','audio');
$type = in_array($type, $types) ? $type : 'image';

$result = array('error' => 1, 'message' => '');

$option = array();
if(isset($_GPC['options'])){
	$option = @base64_decode($_GPC['options']);
	$option = @iunserializer($option);
}

if ($do == 'upload') {
	if($type == 'image'){
		
		if (!isset($option['width'])) {
			$option['width'] = 600;
		}
		if (!empty($option['global']) && empty($_W['isfounder'])) {
			$result['message'] = '没有向 global 文件夹上传图片的权限.';
			frameCallback($_GPC['callback'], json_encode($result));
			exit;
		}
		if (!empty($_FILES['file']['name'])) {
			if ($_FILES['file']['error'] != 0) {
				$result['message'] = '上传失败，请重试！';
				frameCallback($_GPC['callback'], json_encode($result));
				exit;
			}
			$_W['uploadsetting'] = array();
			$_W['uploadsetting']['image']['folder'] = 'images/' . $_W['uniacid'];
			$_W['uploadsetting']['image']['extentions'] = $_W['config']['upload']['image']['extentions'];
			$_W['uploadsetting']['image']['limit'] = $_W['config']['upload']['image']['limit'];
			if (isset($option['global']) && !empty($option['global'])) {
				$_W['uploadsetting']['image']['folder'] = 'images/global';
			}
			load()->func('file');
			$file = file_upload($_FILES['file'], 'image');
			if (is_error($file)) {
				$result['message'] = $file['message'];
				frameCallback($_GPC['callback'], json_encode($result));
				exit;
			}
			
			$path = IA_ROOT .'/'. $_W['config']['upload']['attachdir'].'/';
			$srcfile = $path . $file['path'];
			
			$extention = pathinfo($srcfile, PATHINFO_EXTENSION);
			do {
				if(!empty($option['global'])){
					$filename = "{$_W['uploadsetting']['image']['folder']}/" . random(30) . ".{$extention}";
				} else {
					$filename = "{$_W['uploadsetting']['image']['folder']}/" . date('Y/m/'). random(30) . ".{$extention}";
				}
			} while(file_exists($path .$result['path']. $filename));
			$result['path'] .= $filename;
			$r = file_image_thumb($srcfile, $path . $result['path'], $option['width']);
			
			@unlink($srcfile);
			if (is_error($r)) {
				$result['message'] = $r['message'];
				frameCallback($_GPC['callback'], json_encode($result));
				exit;
			}
			
			$result['filename'] = $result['path'];
			$result['url'] = $_W['attachurl'].$result['path'];
			$result['error'] = 0;
			
			pdo_insert('core_attachment', array(
				'uniacid' => $_W['uniacid'],
				'uid' => $_W['uid'],
				'filename' => $_FILES['file']['name'],
				'attachment' => $result['filename'],
				'type' => 1,
				'createtime' => TIMESTAMP,
			));
			frameCallback($_GPC['callback'], json_encode($result));
			exit;
		} else {
			$result['message'] = '请选择要上传的图片！';
			frameCallback($_GPC['callback'], json_encode($result));
			exit;
		}
				exit;
	} elseif($type == 'audio'){
		
		if (!empty($_FILES['file']['name'])) {
			if ($_FILES['file']['error'] != 0) {
				$result['message'] = '上传失败，请重试！';
				frameCallback($_GPC['callback'], json_encode($result));
				exit;
			}
			$_W['uploadsetting'] = array();
			$_W['uploadsetting']['audio']['folder'] = 'audios/' . $_W['weid'];
			$_W['uploadsetting']['audio']['extentions'] = $_W['config']['upload']['audio']['extentions'];
			$_W['uploadsetting']['audio']['limit'] = $_W['config']['upload']['audio']['limit'];
			load()->func('file');
			$file = file_upload($_FILES['file'], 'audio');
			if (is_error($file)) {
				$result['message'] = $file['message'];
				frameCallback($_GPC['callback'], json_encode($result));
				exit;
			}
			
			$result['path'] .= $file['path'];
			$result['error'] = 0;
			$result['filename'] = $result['path'];
			$result['url'] = $_W['attachurl'].$result['path'];
			
			pdo_insert('core_attachment', array(
				'uniacid' => $_W['uniacid'],
				'uid' => $_W['uid'],
				'filename' => $_FILES['file']['name'],
				'attachment' => $result['filename'],
				'type' => 2,
				'createtime' => TIMESTAMP,
			));
			frameCallback($_GPC['callback'], json_encode($result));
		} else {
			$result['message'] = '请选择要上传的音乐！';
			frameCallback($_GPC['callback'], json_encode($result));
		}
		exit;
	} } 
if ($do == 'browser') {
	
	function file_compare($a, $b) {
		if ($a['is_dir'] && !$b['is_dir']) {
			return -1;
		} elseif(!$a['is_dir'] && $b['is_dir']) {
			return 1;
		} elseif($a['is_dir'] && $b['is_dir']) {
			return strcmp($a['filename'], $b['filename']);
		} else {
			return $a['datetime'] < $b['datetime'] ? -1 : 1;
		}
	}
	
	if($type == 'image') {
		
		$path = $_GPC['path'];
		
		if(!empty($option['global'])){
			if(empty($path)){
				$rootpath = IA_ROOT .'/'.$_W['config']['upload']['attachdir'].'/'.'images/global';
				$currentpath = $rootpath;
				$currentimage = '';
			}else{
				$rootpath = IA_ROOT .'/'.$_W['config']['upload']['attachdir'].'/'.'images/global';
				$currentpath = $rootpath;
				$currentimage = str_replace('images/global/','',$_GPC['path']);
			}
		} else {
			$browser = 'images/'.$_W['uniacid'];
			$parentbrowser = 'images/'.$_W['uniacid'];
			
			if(empty($path)){
				$rootpath = IA_ROOT .'/'.$_W['config']['upload']['attachdir'].'/'.'images/'.$_W['uniacid'];
				$currentpath = $rootpath . $path;
				$currentimage = '';
			} else {
				$strs = explode('/', $path);
				if(isset($strs[0])){
					$imag = intval($strs[0]);
				}
				if(isset($strs[1])){
					$weid = intval($strs[1]);
				}
				if($weid == $_W['uniacid']){
					if(isset($strs[2])){
						$year = strval($strs[2]);
					}
					if(isset($strs[3])){
						$month = strval($strs[3]);
					}
					if(isset($strs[4])){
						$currentimage = strval($strs[4]);
					}
				}
				
				$rootpath = IA_ROOT .'/'.$_W['config']['upload']['attachdir'].'/'.'images/'.$_W['uniacid'];
				$currentpath = $rootpath;
				if(!empty($year)){
					$currentpath .= '/'.$year;
					$browser .= '/'.$year;
				}
				if(!empty($month)){
					$currentpath .= '/'.$month;
					$browser .= '/'.$month;
					$parentbrowser .= '/'.$year;
				}
			}
		}
		$exts = array('gif', 'jpg', 'jpeg', 'png', 'bmp');
		
				$files = array();
		if(empty($option['global'])){
			$files[] = array(
				'filename' => '..',
				'is_dir' => true,
				'datetime' => date('Y-m-d H:i:s', filemtime($file)),
			);
		}
		
		if (is_dir($currentpath)) {
			if ($handle = opendir($currentpath)) {
				while (false !== ($filename = readdir($handle))) {
					if($filename == '.') continue;
					if($filename == '..') continue;
					$file = $currentpath .'/'. $filename;
					
					if (is_dir($file)) {
						if(!empty($option['global'])){
							continue;
						}
						$files[] = array(
							'filename' => $filename,
							'is_dir' => true,
							'datetime' => date('Y-m-d H:i:s', filemtime($file)),
						);
					} else {
						$fileext = strtolower(pathinfo($file, PATHINFO_EXTENSION));
						$entry = array();
						$entry['url'] = $_W['attachurl'].'images/'.$_W['uniacid'].'/'.$year.'/'.$month.'/'.$filename;
						$entry['filename'] = 'images/' . $_W['uniacid'] . '/'.$year.'/'.$month.'/' . $filename;
						if($option['global']){
							$entry['url'] = $_W['attachurl'] . 'images/global/'. $filename;
							$entry['filename'] = 'images/global/'. $filename;
						}
						$files[] = array(
							'filename' => $filename,
							'is_dir' => false,
							'is_photo' => in_array($fileext, $exts),
							'filesize' => filesize($file),
							'filetype' => $fileext,
							'url' => $entry['url'],
							'attachment' => $entry['filename'],
							'entry' => str_replace('"', '\'', json_encode($entry)),
							'datetime' => date('Y-m-d H:i:s', filemtime($file)),
						);
					}
				}
			}
		}
		usort($files, 'file_compare');
		$callback = $_GPC['callback'];
		template('utility/file-browser');
		exit;
	}
	
	if($type == 'audio') {
		
		$path = $_GPC['path'];
		$path = str_replace('..', '', $path);
		$path = str_replace('//', '', $path);
		$path = rtrim($path, '/');
		$path .= '/';
		
		$rootpath = IA_ROOT .'/'.$_W['config']['upload']['attachdir'].'/'.'audios/'.$_W['uniacid'];
		$exts = array('mp3');
		
		$currentpath = $rootpath . $path;
		if ($path == '/') {
			$currentpath = $rootpath . $path;
		}
		
				$files = array();
		if (is_dir($currentpath)) {
			if ($handle = opendir($currentpath)) {
				while (false !== ($filename = readdir($handle))) {
					if($filename == '.') continue;
					if($path == '/' && $filename == '..') continue;
					$file = $currentpath . $filename;
					if (is_dir($file)) {
						$files[] = array(
							'filename' => $filename,
							'is_dir' => true,
							'datetime' => date('Y-m-d H:i:s', filemtime($file)),
						);
					} else {
						$fileext = strtolower(pathinfo($file, PATHINFO_EXTENSION));
						$entry = array();
						$entry['url'] = $_W['attachurl'] . 'audios/' . $_W['uniacid'] . '' . $path . $filename;
						$entry['filename'] = 'audios/' . $_W['uniacid'] . '' . $path . $filename;
						$files[] = array(
							'filename' => $filename,
							'is_dir' => false,
							'is_photo' => in_array($fileext, $exts),
							'filesize' => filesize($file),
							'filetype' => $fileext,
							'url' => $entry['url'],
							'attachment'=>$entry['filename'],
							'entry' => str_replace('"', '\'', json_encode($entry)),
							'datetime' => date('Y-m-d H:i:s', filemtime($file)),
						);
					}
				}
			}
		}
		usort($files, 'file_compare');
		$callback = $_GPC['callback'];
		template('utility/file-browser');
		exit;
	}
}

if ($do == 'delete') {
	
	if (empty($_GPC['file'])) {
		$result['message'] = '请选择要删除的图片！';
		exit(json_encode($result));
	}
	if (empty($_W['isfounder']) && !empty($option['global'])) {
		$result['message'] = '没有删除 global 文件夹中图片的权限.';
		exit(json_encode($result));
	}
	$attachment = $_GPC['file'];
	load()->func('file');
	file_delete($attachment);
	if(empty($option['global'])){
		pdo_delete('core_attachment', array('uniacid'=>$_W['uniacid'], 'attachment'=>$attachment));
	}else{
		pdo_delete('core_attachment', array('attachment'=>$attachment));
	}
	exit('success');
}

function frameCallback($callback, $val) {
	echo '<script type="text/javascript">window.parent.' . $callback . '(' . $val . ');</script>';
	exit;
}