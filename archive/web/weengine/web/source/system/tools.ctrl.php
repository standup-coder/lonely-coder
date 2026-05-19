<?php 
/**
 * [WeEngine System] Copyright (c) 2014 WE7.CC
 * WeEngine is NOT a free software, it under the license terms, visited http://www.we7.cc/ for more details.
 */
$_W['page']['title'] = '检测系统BOM - 工具 - 系统管理';
$dos = array('bom');
$do = in_array($do, $dos) ? $do : 'bom';

if($do == 'bom') {
	if(checksubmit()) {
		set_time_limit(0);
		load()->func('file');
		$path = IA_ROOT;
		$tree = file_tree($path);
		$ds = array();
		foreach($tree as $t) {
			$t = str_replace($path, '', $t);
			$t = str_replace('\\', '/', $t);
			if(preg_match('/^.*\.php$/', $t)) {
				$fname = $path . $t;
				$fp = fopen($fname, 'r');
				if(!empty($fp)) {
					$bom = fread($fp, 3);
					fclose($fp);
					if($bom == "\xEF\xBB\xBF") {
						$ds[] = $t;
					}
				}
			}
		}
	}
}

template('system/bom');