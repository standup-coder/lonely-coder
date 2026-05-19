define(['bootstrap'], function($){
	var module = {};
	
	module.tomedia = function(src){
		if(src.indexOf('http://') == 0 || src.indexOf('https://') == 0 || src.indexOf('./resource') == 0) {
			return src;
		} else if(src.indexOf('./addons') == 0) {
			var url=window.document.location.href; 
			var pathName = window.document.location.pathname; 
			var pos = url.indexOf(pathName); 
			var host = url.substring(0,pos);
			if (src.substr(0,1)=='.') {
				src=src.substr(1);
			}
			return host + src;
		} else {
			return '../attachment/' + src;
		}
	};
	
	module.dialog = function(title, content, footer, options) {
		if(!options) {
			options = {};
		}
		if(!options.containerName) {
			options.containerName = 'modal-message';
		}
		var modalobj = $('#' + options.containerName);
		if(modalobj.length == 0) {
			$(document.body).append('<div id="' + options.containerName + '" class="modal animated" tabindex="-1" role="dialog" aria-hidden="true"></div>');
			modalobj = $('#' + options.containerName);
		}
		var html =
			'<div class="modal-dialog modal-sm">'+
			'	<div class="modal-content">';
		if(title) {
			html +=
			'<div class="modal-header">'+
			'	<button type="button" class="close" data-dismiss="modal" aria-hidden="true">×</button>'+
			'	<h3>' + title + '</h3>'+
			'</div>';
		}
		if(content) {
			if(!$.isArray(content)) {
				html += '<div class="modal-body">'+ content + '</div>';
			} else {
				html += '<div class="modal-body">正在加载中</div>';
			}
		}
		if(footer) {
			html +=
			'<div class="modal-footer">'+ footer + '</div>';
		}
		html += '	</div></div>';
		modalobj.html(html);
		if(content && $.isArray(content)) {
			var embed = function(c) {
				modalobj.find('.modal-body').html(c);
			};
			if(content.length == 2) {
				$.post(content[0], content[1]).success(embed);
			} else {
				$.get(content[0]).success(embed);
			}
		}
		return modalobj;
	};
	
	module.message = function(msg, redirect, type){
		if(!redirect && !type){
			type = 'info';
		}
		if($.inArray(type, ['success', 'error', 'info', 'warning']) == -1) {
			type = '';
		}
		if(type == '') {
			type = redirect == '' ? 'error' : 'success';
		}
		
		var icons = {
			success : 'check-circle',
			error :'times-circle',
			info : 'info-circle',
			warning : 'exclamation-triangle'
		};
		var p = '';
		if(redirect && redirect.length > 0){
			if(redirect == 'back'){
				p = '<p>[<a href="javascript:;" onclick="history.go(-1)">返回上一页</a>] &nbsp; [<a href="./?refresh">回首页</a>]</p>';
			}else{
				p = '<p><a href="' + redirect + '" target="main" data-dismiss="modal" aria-hidden="true">如果你的浏览器在 <span id="timeout"></span> 秒后没有自动跳转，请点击此链接</a></p>';
			}
		}
		var content = 
			'			<i class="pull-left fa fa-4x fa-'+icons[type]+'"></i>'+
			'			<div class="pull-left"><p>'+ msg +'</p>' +
			p +
			'			</div>'+
			'			<div class="clearfix"></div>';
		var footer = 
			'			<button type="button" class="btn btn-default" data-dismiss="modal">确认</button>';
		var modalobj = module.dialog('系统提示', content, footer);
		modalobj.find('.modal-content').addClass('alert alert-'+type);
		if(redirect) {
			var timer = 0;
			var timeout = 3;
			modalobj.find("#timeout").html(timeout);
			modalobj.on('show.bs.modal', function(){doredirect();});
			modalobj.on('hide.bs.modal', function(){timeout = 0;doredirect(); });
			modalobj.on('hidden.bs.modal', function(){modalobj.remove();});
			function doredirect() {
				timer = setTimeout(function(){
					if (timeout <= 0) {
						modalobj.modal('hide');
						clearTimeout(timer);
						window.location.href = redirect;
						return;
					} else {
						timeout--;
						modalobj.find("#timeout").html(timeout);
						doredirect();
					}
				}, 1000);
			}
		}
		modalobj.on('show.bs.modal', function(e){
			$(e.target).removeClass('bounceOut');
			$(e.target).addClass('bounceIn');
		})
		modalobj.on('hide.bs.modal', function(e){
			if(!e.target.animated) {
				$(e.target).removeClass('bounceIn');
				$(e.target).addClass('bounceOut');
				e.preventDefault();
				e.target.animated = true;
				setTimeout(function(){
					$(e.target).modal('hide');
					e.target.animated = false;
				}, 1000);
			}
		})
		modalobj.modal('show');
	};

	/**
	 * 点击指定的元素, 发送验证码, 并显示倒计时, 并通知发送状态
	 * @param elm 元素节点
	 * @param no 要发送验证码的手机号
	 * @param callback 通知回调, 这个函数接受两个参数
	 * function(ret, state)
	 * ret 通知结果, success 成功, failed 失败, downcount 倒计时
	 * state 通知内容, success 时无数据, failed 时指明失败原因, downcount 时指明当前倒数
	 */
	module.sendCode = function(elm, no, callback) {
		if(!no || !elm || !$(elm).attr('uniacid')) {
			if($.isFunction(callback)) {
				callback('failed', '给定的参数有错误');
			}
			return;
		}
		$(elm).attr("disabled", true);
		var downcount = 60;
		$(elm).html(downcount + "秒后重新获取");

		var timer = setInterval(function(){
			downcount--;
			if(downcount <= 0){
				clearInterval(timer);
				$(elm).html("重新获取验证码");
				$(elm).attr("disabled", false);
				downcount = 60;
			}else{
				if($.isFunction(callback)) {
					callback('downcount', downcount);
				}
				$(elm).html(downcount + "秒后重新获取");
			}
		}, 1000);

		var params = {};
		params.receiver = no;
		params.uniacid = $(elm).attr('uniacid');
		$.post('../web/index.php?c=utility&a=verifycode', params).success(function(dat){
			if(dat == 'success') {
				if($.isFunction(callback)) {
					callback('success', null);
				}
			} else {
				if($.isFunction(callback)) {
					callback('failed', dat);
				}
			}
		});
	};
	
	module.image = function(val, callback, opts) {
		var content = 	'<div class = "tab-content form-horizontal" style="padding:20px 0;">'+
							'<div class="form-group tab-pane active" id="image_upload">'+
								'<iframe width="0" height="0" name="__image_file_uploader" style="display:none;"></iframe>' +
								'<form action="./index.php?c=utility&a=file&do=upload&type=image&callback=uploaderImageComplete&i='+window['__uniacid']+'" enctype="multipart/form-data" method="post" target="__image_file_uploader">'+
									'<div class="form-group">' +
										'<div class="col-sm-12">' +
											'<input type="file" name="file">'+
											'<input type="hidden" name="options">'+
										'</div>' +
									'</div>' +
								'</form>' +
							'</div>' +
						'</div>';
		var footer = 
			'<button type="button" class="btn btn-default" data-dismiss="modal">取消</button>' +
			'<button type="button" class="btn btn-primary">确认</button>';
		var modalobj = module.dialog('请上传图片', content, footer, {containerName: 'image-container'}); 
		
		modalobj.modal({'keyboard': false});
		modalobj.find(':hidden[name="options"]').val(opts);
		modalobj.find('button.btn-primary').off('click');
		modalobj.find('button.btn-primary').on('click', function(){
			if(modalobj.find('.nav.nav-tabs li').eq(0).hasClass('active')) {
				var url = modalobj.find('#image_url').val();
				
				var reg = /^images\/[\d]+\/[\d]+\/[\d]+\/[\S]+/i;
				if(reg.test(url)){
					callback({filename: url, url: module.tomedia(url)});
					modalobj.modal('hide');
					return;
				}
				reg = /^images\/global\/[\S]+/i;
				if(reg.test(url)){
					callback({filename: url, url: module.tomedia(url)});
					modalobj.modal('hide');
					return;
				}
				var httpreg = /^http:\/\/[^\S]*/i;
				if(httpreg.test(url)){
					callback({filename: module.tomedia(url), url: module.tomedia(url)});
					modalobj.modal('hide');
				}
			} else {
				modalobj.find('form')[0].submit();
			}
		});
		require(['filestyle'], function($){
			modalobj.find(':file[name="file"]').filestyle({buttonText: '上传图片'});
		});
		window.uploaderImageComplete = function(r){
			if(r && r.filename && r.url) {
				callback({filename: r.filename, url: r.url});
				modalobj.modal('hide');
			} else {
				module.message(r.message,'','error');
			}
		};
	}; // end of image
	
	return module;
});

