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

	module.clip = function(elm, str) {
		if(elm.clip) {
			return;
		}
		require(['jquery.zclip'], function(){
			$(elm).zclip({
				path: './resource/components/zclip/ZeroClipboard.swf',
				copy: str,
				afterCopy: function(){
					var obj = $('<em> &nbsp; <span class="label label-success"><i class="fa fa-check-circle"></i> 复制成功</span></em>');
					var enext = $(elm).next().html();
					if (!enext || enext.indexOf('&nbsp; <span class="label label-success"><i class="fa fa-check-circle"></i> 复制成功</span>')<0) {
						$(elm).after(obj);
					}
					setTimeout(function(){
						obj.remove();
					}, 2000);
				}
			});
			elm.clip = true;
		});
	};
	
	module.colorpicker = function(elm, callback) {
		require(['colorpicker'], function(){
			$(elm).spectrum({
				className : "colorpicker",
				showInput: true,
				showInitial: true,
				showPalette: true,
				maxPaletteSize: 10,
				preferredFormat: "hex",
				change: function(color) {
					if($.isFunction(callback)) {
						callback(color);
					}
				},
				palette: [
					["rgb(0, 0, 0)", "rgb(67, 67, 67)", "rgb(102, 102, 102)", "rgb(153, 153, 153)","rgb(183, 183, 183)",
					"rgb(204, 204, 204)", "rgb(217, 217, 217)","rgb(239, 239, 239)", "rgb(243, 243, 243)", "rgb(255, 255, 255)"],
					["rgb(152, 0, 0)", "rgb(255, 0, 0)", "rgb(255, 153, 0)", "rgb(255, 255, 0)", "rgb(0, 255, 0)",
					"rgb(0, 255, 255)", "rgb(74, 134, 232)", "rgb(0, 0, 255)", "rgb(153, 0, 255)", "rgb(255, 0, 255)"],
					["rgb(230, 184, 175)", "rgb(244, 204, 204)", "rgb(252, 229, 205)", "rgb(255, 242, 204)", "rgb(217, 234, 211)",
					"rgb(208, 224, 227)", "rgb(201, 218, 248)", "rgb(207, 226, 243)", "rgb(217, 210, 233)", "rgb(234, 209, 220)",
					"rgb(221, 126, 107)", "rgb(234, 153, 153)", "rgb(249, 203, 156)", "rgb(255, 229, 153)", "rgb(182, 215, 168)",
					"rgb(162, 196, 201)", "rgb(164, 194, 244)", "rgb(159, 197, 232)", "rgb(180, 167, 214)", "rgb(213, 166, 189)",
					"rgb(204, 65, 37)", "rgb(224, 102, 102)", "rgb(246, 178, 107)", "rgb(255, 217, 102)", "rgb(147, 196, 125)",
					"rgb(118, 165, 175)", "rgb(109, 158, 235)", "rgb(111, 168, 220)", "rgb(142, 124, 195)", "rgb(194, 123, 160)",
					"rgb(166, 28, 0)", "rgb(204, 0, 0)", "rgb(230, 145, 56)", "rgb(241, 194, 50)", "rgb(106, 168, 79)",
					"rgb(69, 129, 142)", "rgb(60, 120, 216)", "rgb(61, 133, 198)", "rgb(103, 78, 167)", "rgb(166, 77, 121)",
					"rgb(133, 32, 12)", "rgb(153, 0, 0)", "rgb(180, 95, 6)", "rgb(191, 144, 0)", "rgb(56, 118, 29)",
					"rgb(19, 79, 92)", "rgb(17, 85, 204)", "rgb(11, 83, 148)", "rgb(53, 28, 117)", "rgb(116, 27, 71)",
					"rgb(91, 15, 0)", "rgb(102, 0, 0)", "rgb(120, 63, 4)", "rgb(127, 96, 0)", "rgb(39, 78, 19)",
					"rgb(12, 52, 61)", "rgb(28, 69, 135)", "rgb(7, 55, 99)", "rgb(32, 18, 77)", "rgb(76, 17, 48)"]
				]
			});
		});
	}
	
	module.uploadMultiPictures = function(callback){
		require(['jquery', 'kindeditor'], function($){
			var editor = KindEditor.editor({
				allowFileManager : false,
				imageSizeLimit : '30MB',
				uploadJson : './index.php?c=utility&a=file&do=upload&type=image&ajax=1'
			});
			editor.loadPlugin('multiimage', function() {
				editor.plugin.multiImageDialog({
					clickFn : function(list) {
						if (list && list.length > 0) {
							if($.isFunction(callback)) {
								callback(list);
							}
							editor.hideDialog();
						} else {
							alert('请先选择要上传的图片！');
						}
					}
				});
			});
		});
	}
	
	module.editor = function(elm, callback){
		var id = elm.id;
		if(!id) {
			id = 'editor-' + Math.random();
			elm.id = id;
		}
		if(!elm.editor) {
			require(['editor'], function(){
				var editor = tinyMCE.createEditor(id, {
					plugins: [
						"advlist autolink lists link image charmap print preview hr anchor pagebreak",
						"searchreplace wordcount visualblocks visualchars code fullscreen",
						"insertdatetime media nonbreaking save table contextmenu directionality",
						"emoticons template paste textcolor"
					],
					toolbar1: "undo redo | bold italic | forecolor backcolor | alignleft aligncenter alignright alignjustify | bullist numlist outdent indent | preview fullscreen",
					toolbar2: "code print | styleselect link image media emoticons",
					language: 'zh_CN',
					menubar: false
				});
				elm.editor = editor;
				editor.render();
				if($.isFunction(callback)) {
					callback(elm, editor);
				}
			});
		}
		return {
			getContent : function(){
				if(elm.editor) {
					return elm.editor.getContent();
				} else {
					return '';
				}
			}
		};
	};
	// target dom 对象
	module.emotion = function(elm, target, callback) {
		require(['jquery.caret', 'bootstrap', 'css!../../components/emotions/emotions.css'],function($){
			$(function() {
				var emotions_html = '<table class="emotions" cellspacing="0" cellpadding="0"><tbody><tr><td><div class="eItem" style="background-position:0px 0;" data-title="微笑" data-code="::)" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/0.gif"></div></td><td><div class="eItem" style="background-position:-24px 0;" data-title="撇嘴" data-code="::~" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/1.gif"></div></td><td><div class="eItem" style="background-position:-48px 0;" data-title="色" data-code="::B" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/2.gif"></div></td><td><div class="eItem" style="background-position:-72px 0;" data-title="发呆" data-code="::|" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/3.gif"></div></td><td><div class="eItem" style="background-position:-96px 0;" data-title="得意" data-code=":8-)" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/4.gif"></div></td><td><div class="eItem" style="background-position:-120px 0;" data-title="流泪" data-code="::<" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/5.gif"></div></td><td><div class="eItem" style="background-position:-144px 0;" data-title="害羞" data-code="::$" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/6.gif"></div></td><td><div class="eItem" style="background-position:-168px 0;" data-title="闭嘴" data-code="::X" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/7.gif"></div></td><td><div class="eItem" style="background-position:-192px 0;" data-title="睡" data-code="::Z" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/8.gif"></div></td><td><div class="eItem" style="background-position:-216px 0;" data-title="大哭" data-code="::\'(" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/9.gif"></div></td><td><div class="eItem" style="background-position:-240px 0;" data-title="尴尬" data-code="::-|" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/10.gif"></div></td><td><div class="eItem" style="background-position:-264px 0;" data-title="发怒" data-code="::@" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/11.gif"></div></td><td><div class="eItem" style="background-position:-288px 0;" data-title="调皮" data-code="::P" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/12.gif"></div></td><td><div class="eItem" style="background-position:-312px 0;" data-title="呲牙" data-code="::D" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/13.gif"></div></td><td><div class="eItem" style="background-position:-336px 0;" data-title="惊讶" data-code="::O" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/14.gif"></div></td></tr><tr><td><div class="eItem" style="background-position:-360px 0;" data-title="难过" data-code="::(" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/15.gif"></div></td><td><div class="eItem" style="background-position:-384px 0;" data-title="酷" data-code="::+" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/16.gif"></div></td><td><div class="eItem" style="background-position:-408px 0;" data-title="冷汗" data-code=":--b" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/17.gif"></div></td><td><div class="eItem" style="background-position:-432px 0;" data-title="抓狂" data-code="::Q" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/18.gif"></div></td><td><div class="eItem" style="background-position:-456px 0;" data-title="吐" data-code="::T" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/19.gif"></div></td><td><div class="eItem" style="background-position:-480px 0;" data-title="偷笑" data-code=":,@P" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/20.gif"></div></td><td><div class="eItem" style="background-position:-504px 0;" data-title="可爱" data-code=":,@-D" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/21.gif"></div></td><td><div class="eItem" style="background-position:-528px 0;" data-title="白眼" data-code="::d" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/22.gif"></div></td><td><div class="eItem" style="background-position:-552px 0;" data-title="傲慢" data-code=":,@o" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/23.gif"></div></td><td><div class="eItem" style="background-position:-576px 0;" data-title="饥饿" data-code="::g" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/24.gif"></div></td><td><div class="eItem" style="background-position:-600px 0;" data-title="困" data-code=":|-)" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/25.gif"></div></td><td><div class="eItem" style="background-position:-624px 0;" data-title="惊恐" data-code="::!" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/26.gif"></div></td><td><div class="eItem" style="background-position:-648px 0;" data-title="流汗" data-code="::L" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/27.gif"></div></td><td><div class="eItem" style="background-position:-672px 0;" data-title="憨笑" data-code="::>" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/28.gif"></div></td><td><div class="eItem" style="background-position:-696px 0;" data-title="大兵" data-code="::,@" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/29.gif"></div></td></tr><tr><td><div class="eItem" style="background-position:-720px 0;" data-title="奋斗" data-code=":,@f" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/30.gif"></div></td><td><div class="eItem" style="background-position:-744px 0;" data-title="咒骂" data-code="::-S" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/31.gif"></div></td><td><div class="eItem" style="background-position:-768px 0;" data-title="疑问" data-code=":?" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/32.gif"></div></td><td><div class="eItem" style="background-position:-792px 0;" data-title="嘘" data-code=":,@x" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/33.gif"></div></td><td><div class="eItem" style="background-position:-816px 0;" data-title="晕" data-code=":,@@" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/34.gif"></div></td><td><div class="eItem" style="background-position:-840px 0;" data-title="折磨" data-code="::8" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/35.gif"></div></td><td><div class="eItem" style="background-position:-864px 0;" data-title="衰" data-code=":,@!" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/36.gif"></div></td><td><div class="eItem" style="background-position:-888px 0;" data-title="骷髅" data-code=":!!!" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/37.gif"></div></td><td><div class="eItem" style="background-position:-912px 0;" data-title="敲打" data-code=":xx" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/38.gif"></div></td><td><div class="eItem" style="background-position:-936px 0;" data-title="再见" data-code=":bye" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/39.gif"></div></td><td><div class="eItem" style="background-position:-960px 0;" data-title="擦汗" data-code=":wipe" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/40.gif"></div></td><td><div class="eItem" style="background-position:-984px 0;" data-title="抠鼻" data-code=":dig" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/41.gif"></div></td><td><div class="eItem" style="background-position:-1008px 0;" data-title="鼓掌" data-code=":handclap" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/42.gif"></div></td><td><div class="eItem" style="background-position:-1032px 0;" data-title="糗大了" data-code=":&-(" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/43.gif"></div></td><td><div class="eItem" style="background-position:-1056px 0;" data-title="坏笑" data-code=":B-)" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/44.gif"></div></td></tr><tr><td><div class="eItem" style="background-position:-1080px 0;" data-title="左哼哼" data-code=":<@" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/45.gif"></div></td><td><div class="eItem" style="background-position:-1104px 0;" data-title="右哼哼" data-code=":@>" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/46.gif"></div></td><td><div class="eItem" style="background-position:-1128px 0;" data-title="哈欠" data-code="::-O" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/47.gif"></div></td><td><div class="eItem" style="background-position:-1152px 0;" data-title="鄙视" data-code=":>-|" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/48.gif"></div></td><td><div class="eItem" style="background-position:-1176px 0;" data-title="委屈" data-code=":P-(" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/49.gif"></div></td><td><div class="eItem" style="background-position:-1200px 0;" data-title="快哭了" data-code="::\'|" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/50.gif"></div></td><td><div class="eItem" style="background-position:-1224px 0;" data-title="阴险" data-code=":X-)" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/51.gif"></div></td><td><div class="eItem" style="background-position:-1248px 0;" data-title="亲亲" data-code="::*" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/52.gif"></div></td><td><div class="eItem" style="background-position:-1272px 0;" data-title="吓" data-code=":@x" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/53.gif"></div></td><td><div class="eItem" style="background-position:-1296px 0;" data-title="可怜" data-code=":8*" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/54.gif"></div></td><td><div class="eItem" style="background-position:-1320px 0;" data-title="菜刀" data-code=":pd" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/55.gif"></div></td><td><div class="eItem" style="background-position:-1344px 0;" data-title="西瓜" data-code=":<W>" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/56.gif"></div></td><td><div class="eItem" style="background-position:-1368px 0;" data-title="啤酒" data-code=":beer" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/57.gif"></div></td><td><div class="eItem" style="background-position:-1392px 0;" data-title="篮球" data-code=":basketb" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/58.gif"></div></td><td><div class="eItem" style="background-position:-1416px 0;" data-title="乒乓" data-code=":oo" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/59.gif"></div></td></tr><tr><td><div class="eItem" style="background-position:-1440px 0;" data-title="咖啡" data-code=":coffee" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/60.gif"></div></td><td><div class="eItem" style="background-position:-1464px 0;" data-title="饭" data-code=":eat" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/61.gif"></div></td><td><div class="eItem" style="background-position:-1488px 0;" data-title="猪头" data-code=":pig" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/62.gif"></div></td><td><div class="eItem" style="background-position:-1512px 0;" data-title="玫瑰" data-code=":rose" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/63.gif"></div></td><td><div class="eItem" style="background-position:-1536px 0;" data-title="凋谢" data-code=":fade" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/64.gif"></div></td><td><div class="eItem" style="background-position:-1560px 0;" data-title="示爱" data-code=":showlove" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/65.gif"></div></td><td><div class="eItem" style="background-position:-1584px 0;" data-title="爱心" data-code=":heart" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/66.gif"></div></td><td><div class="eItem" style="background-position:-1608px 0;" data-title="心碎" data-code=":break" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/67.gif"></div></td><td><div class="eItem" style="background-position:-1632px 0;" data-title="蛋糕" data-code=":cake" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/68.gif"></div></td><td><div class="eItem" style="background-position:-1656px 0;" data-title="闪电" data-code=":li" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/69.gif"></div></td><td><div class="eItem" style="background-position:-1680px 0;" data-title="炸弹" data-code=":bome" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/70.gif"></div></td><td><div class="eItem" style="background-position:-1704px 0;" data-title="刀" data-code=":kn" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/71.gif"></div></td><td><div class="eItem" style="background-position:-1728px 0;" data-title="足球" data-code=":footb" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/72.gif"></div></td><td><div class="eItem" style="background-position:-1752px 0;" data-title="瓢虫" data-code=":ladybug" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/73.gif"></div></td><td><div class="eItem" style="background-position:-1776px 0;" data-title="便便" data-code=":shit" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/74.gif"></div></td></tr><tr><td><div class="eItem" style="background-position:-1800px 0;" data-title="月亮" data-code=":moon" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/75.gif"></div></td><td><div class="eItem" style="background-position:-1824px 0;" data-title="太阳" data-code=":sun" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/76.gif"></div></td><td><div class="eItem" style="background-position:-1848px 0;" data-title="礼物" data-code=":gift" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/77.gif"></div></td><td><div class="eItem" style="background-position:-1872px 0;" data-title="拥抱" data-code=":hug" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/78.gif"></div></td><td><div class="eItem" style="background-position:-1896px 0;" data-title="强" data-code=":strong" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/79.gif"></div></td><td><div class="eItem" style="background-position:-1920px 0;" data-title="弱" data-code=":weak" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/80.gif"></div></td><td><div class="eItem" style="background-position:-1944px 0;" data-title="握手" data-code=":share" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/81.gif"></div></td><td><div class="eItem" style="background-position:-1968px 0;" data-title="胜利" data-code=":v" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/82.gif"></div></td><td><div class="eItem" style="background-position:-1992px 0;" data-title="抱拳" data-code=":@)" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/83.gif"></div></td><td><div class="eItem" style="background-position:-2016px 0;" data-title="勾引" data-code=":jj" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/84.gif"></div></td><td><div class="eItem" style="background-position:-2040px 0;" data-title="拳头" data-code=":@@" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/85.gif"></div></td><td><div class="eItem" style="background-position:-2064px 0;" data-title="差劲" data-code=":bad" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/86.gif"></div></td><td><div class="eItem" style="background-position:-2088px 0;" data-title="爱你" data-code=":lvu" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/87.gif"></div></td><td><div class="eItem" style="background-position:-2112px 0;" data-title="NO" data-code=":no" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/88.gif"></div></td><td><div class="eItem" style="background-position:-2136px 0;" data-title="OK" data-code=":ok" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/89.gif"></div></td></tr><tr><td><div class="eItem" style="background-position:-2160px 0;" data-title="爱情" data-code=":love" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/90.gif"></div></td><td><div class="eItem" style="background-position:-2184px 0;" data-title="飞吻" data-code=":<L>" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/91.gif"></div></td><td><div class="eItem" style="background-position:-2208px 0;" data-title="跳跳" data-code=":jump" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/92.gif"></div></td><td><div class="eItem" style="background-position:-2232px 0;" data-title="发抖" data-code=":shake" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/93.gif"></div></td><td><div class="eItem" style="background-position:-2256px 0;" data-title="怄火" data-code=":<O>" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/94.gif"></div></td><td><div class="eItem" style="background-position:-2280px 0;" data-title="转圈" data-code=":circle" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/95.gif"></div></td><td><div class="eItem" style="background-position:-2304px 0;" data-title="磕头" data-code=":kotow" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/96.gif"></div></td><td><div class="eItem" style="background-position:-2328px 0;" data-title="回头" data-code=":turn" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/97.gif"></div></td><td><div class="eItem" style="background-position:-2352px 0;" data-title="跳绳" data-code=":skip" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/98.gif"></div></td><td><div class="eItem" style="background-position:-2376px 0;" data-title="挥手" data-code=":oY" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/99.gif"></div></td><td><div class="eItem" style="background-position:-2400px 0;" data-title="激动" data-code=":#-0" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/100.gif"></div></td><td><div class="eItem" style="background-position:-2424px 0;" data-title="街舞" data-code=":hiphot" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/101.gif"></div></td><td><div class="eItem" style="background-position:-2448px 0;" data-title="献吻" data-code=":kiss" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/102.gif"></div></td><td><div class="eItem" style="background-position:-2472px 0;" data-title="左太极" data-code=":<&" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/103.gif"></div></td><td><div class="eItem" style="background-position:-2496px 0;" data-title="右太极" data-code=":&>" data-gifurl="http://res.mail.qq.com/zh_CN/images/mo/DEFAULT2/104.gif"></div></td></tr></tbody></table><div class="emotionsGif" style=""></div>';
				$(elm).popover({
					html: true,
					content: emotions_html,
					placement:"bottom"
				});
				$(elm).one('shown.bs.popover', function(){
					$(elm).next().mouseleave(function(){
						$(elm).popover('hide');
					});
					$(elm).next().delegate(".eItem", "mouseover", function(){
						var emo_img = '<img src="'+$(this).attr("data-gifurl")+'" alt="mo-'+$(this).attr("data-title")+'" />';
						var emo_txt = '/'+$(this).attr("data-code");
						$(elm).next().find(".emotionsGif").html(emo_img);
					});
					$(elm).next().delegate(".eItem", "click", function(){
						$(target).setCaret();
						var emo_txt = '/'+$(this).attr("data-code");
						$(target).insertAtCaret(emo_txt);
						$(elm).popover('hide');
						if($.isFunction(callback)) {
							callback(emo_txt, elm, target);
						}
					});
				});
			});
		});
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
			$(document.body).append('<div id="' + options.containerName + '" class="modal fade" tabindex="-1" role="dialog" aria-hidden="true"></div>');
			modalobj = $('#' + options.containerName);
		}
		html = 
			'<div class="modal-dialog">'+
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
		var modalobj = module.dialog('系统提示', content, footer, {'containerName' : 'modal-message'});
		modalobj.find('.modal-content').addClass('alert alert-'+type);
		if(redirect) {
			var timer = '';
			timeout = 3;
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
		modalobj.modal('show');
		return modalobj;
	};
	
	module.map = function(val, callback){
		require(['map'], function(BMap){
			if(!val) {
				val = {};
			}
			if(!val.lng) {
				val.lng = 116.403851;
			}
			if(!val.lat) {
				val.lat = 39.915177;
			}
			var point = new BMap.Point(val.lng, val.lat);
			var geo = new BMap.Geocoder();
			
			var modalobj = $('#map-dialog');
			if(modalobj.length == 0) {
				var content =
					'<div class="form-group">' +
						'<div class="input-group">' +
							'<input type="text" class="form-control" placeholder="请输入地址来直接查找相关位置">' +
							'<div class="input-group-btn">' +
								'<button class="btn btn-default"><i class="icon-search"></i> 搜索</button>' +
							'</div>' +
						'</div>' +
					'</div>' +
					'<div id="map-container" style="height:400px;"></div>';
				var footer =
					'<button type="button" class="btn btn-default" data-dismiss="modal">取消</button>' +
					'<button type="button" class="btn btn-primary">确认</button>';
				modalobj = module.dialog('请选择地点', content, footer, {containerName : 'map-dialog'});
				modalobj.find('.modal-dialog').css('width', '80%');
				modalobj.modal({'keyboard': false});
				
				map = module.map.instance = new BMap.Map('map-container');
				map.centerAndZoom(point, 12);
				map.enableScrollWheelZoom();
				map.enableDragging();
				map.enableContinuousZoom();
				map.addControl(new BMap.NavigationControl());
				map.addControl(new BMap.OverviewMapControl());
				marker = module.map.marker = new BMap.Marker(point);
				marker.setLabel(new BMap.Label('请您移动此标记，选择您的坐标！', {'offset': new BMap.Size(10,-20)}));
				map.addOverlay(marker);
				marker.enableDragging();
				marker.addEventListener('dragend', function(e){
					var point = marker.getPosition();
					geo.getLocation(point, function(address){
						modalobj.find('.input-group :text').val(address.address);
					});
				});
				function searchAddress(address) {
					geo.getPoint(address, function(point){
						map.panTo(point);
						marker.setPosition(point);
						marker.setAnimation(BMAP_ANIMATION_BOUNCE);
						setTimeout(function(){marker.setAnimation(null)}, 3600);
					});
				}
				modalobj.find('.input-group :text').keydown(function(e){
					if(e.keyCode == 13) {
						var kw = $(this).val();
						searchAddress(kw);
					}
				});
				modalobj.find('.input-group button').click(function(){
					var kw = $(this).parent().prev().val();
					searchAddress(kw);
				});
			}
			modalobj.off('shown.bs.modal');
			modalobj.on('shown.bs.modal', function(){
				marker.setPosition(point);
				map.panTo(marker.getPosition());
			});
			
			modalobj.find('button.btn-primary').off('click');
			modalobj.find('button.btn-primary').on('click', function(){
				if($.isFunction(callback)) {
					var point = module.map.marker.getPosition();
					geo.getLocation(point, function(address){
						var val = {lng: point.lng, lat: point.lat, label: address.address};
						callback(val);
					});
				}
				modalobj.modal('hide');
			});
			modalobj.modal('show');
		});
	}; // end of map
	
	module.iconBrowser = function(callback){
		var footer = '<button type="button" class="btn btn-default" data-dismiss="modal">取消</button>';
		var modalobj = module.dialog('请选择图标',['./index.php?c=utility&a=icon&callback=selectIconComplete'],footer,{containerName:'icon-container'});
		modalobj.modal({'keyboard': false});
		modalobj.find('.modal-dialog').css({'width':'70%'});
		modalobj.find('.modal-body').css({'height':'70%','overflow-y':'scroll'});
		modalobj.modal('show');

		window.selectIconComplete = function(ico){
			if($.isFunction(callback)){
				callback(ico);
				modalobj.modal('hide');
			}
		};
	}; // end of icon dialog
	
	module.emojiBrowser = function(callback){
		var footer = '<button type="button" class="btn btn-default" data-dismiss="modal">取消</button>';
		var modalobj = module.dialog('请选择表情',['./index.php?c=utility&a=emoji&callback=selectEmojiComplete'],footer,{containerName:'icon-container'});
		modalobj.modal({'keyboard': false});
		modalobj.find('.modal-dialog').css({'width':'70%'});
		modalobj.find('.modal-body').css({'height':'70%','overflow-y':'scroll'});
		modalobj.modal('show');

		window.selectEmojiComplete = function(emoji){
			if($.isFunction(callback)){
				callback(emoji);
				modalobj.modal('hide');
			}
		};
	}; // end of emoji dialog
	
	module.linkBrowser = function(callback){
		var footer = '<button type="button" class="btn btn-default" data-dismiss="modal">取消</button>';
		var modalobj = module.dialog('请选择链接',['./index.php?c=utility&a=link&callback=selectLinkComplete'],footer,{containerName:'link-container'});
		modalobj.modal({'keyboard': false});
		modalobj.find('.modal-body').css({'height':'300px','overflow-y':'auto' });
		modalobj.modal('show');
		
		window.selectLinkComplete = function(link){
			if($.isFunction(callback)){
				callback(link);
				modalobj.modal('hide');
			}
		};
	}; // end of icon dialog
	
	module.image = function(val, callback, opts) {
		var content = 	'<ul class = "nav nav-tabs" style="margin:auto -10px;padding:0 10px;">'+
							'<li class="active"><a href="#image_browser" data-toggle="tab">网络图片</a></li>'+
							'<li><a href="#image_upload" data-toggle="tab">上传图片</a></li>'+
						'</ul>'+
						'<div class = "tab-content form-horizontal" style="padding:20px 0;">'+
							'<div class="tab-pane active" id="image_browser">'+
								'<div class="form-group">' +
									'<label class="col-xs-12 col-sm-2 control-label">图片地址</label>' +
									'<div class="col-sm-10">' +
										'<div class="input-group">' +
											'<input class="form-control" type="text" id="image_url" value="' + val + '" placeholder="请输入图片URL"/>' +
											'<span class="input-group-btn">' +
												'<button class="btn btn-default btn-browser" type="button">浏览图片空间</button>' +
											'</span>' +
										'</div>' +
									'</div>' +
								'</div>' +
							'</div>'+
							'<div class="tab-pane" id="image_upload">'+
								'<iframe width="0" height="0" name="__image_file_uploader" style="display:none;"></iframe>' +
								'<form action="./index.php?c=utility&a=file&do=upload&type=image&callback=uploaderImageComplete" enctype="multipart/form-data" method="post" target="__image_file_uploader">'+
									'<div class="form-group">' +
										'<label class="col-xs-12 col-sm-2 control-label">上传图片</label>' +
										'<div class="col-sm-10">' +
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
		var modalobj = module.dialog('请选择图片', content, footer, {containerName: 'image-container'}); 
		
		modalobj.modal({'keyboard': false});
		modalobj.find('button.btn-browser').off('click');
		modalobj.find('button.btn-browser').on('click', function(){
			
			var dialog = module.dialog('浏览图片空间的图片', '正在加载', footer, {containerName: 'image-container'});
			dialog.find('.modal-dialog').css('width', '80%');
			dialog.find('.modal-dialog').css('min-height', 600);
			dialog.modal('show');
			
			window.imageBrowser = {
				attachpath : '',
				browser: function(path){
					if(!path){
						path = '';
					}
					dialog.find('.modal-body').load('./index.php?c=utility&a=file&do=browser&options='+opts+'&type=image&callback=imageBrowser&path=' + path);
				},
				select: function(r) {
					callback({filename: r.filename, url: r.url});
					modalobj.modal('hide');
				},
				delete: function(file){
					$.get('./index.php?c=utility&a=file&do=delete&type=image&options='+opts+'&file=' + file).success(function(dat){
						if(dat == 'success') {
							console.log(file);
							window.imageBrowser.browser(file);
						} else {
							var o = $.parseJSON(dat);
							module.message(o.message, '', 'info');
						}
					});
				}
			};
			window.imageBrowser.attachurl = (function(){
				var url=window.document.location.href; 
				var pathName = window.document.location.pathname; 
				var pos = url.indexOf(pathName); 
				var host = url.substring(0,pos);
				return host + '/attachment/';
			})();
			
			val = val.replace(window.imageBrowser.attachurl, '');
			val = val.replace(/(^\s*)|(\s*$)/g,"");
			var reg = /^images[\S]+/i;
			if(reg.test(val)){
				window.imageBrowser.browser(val);
			} else {
				window.imageBrowser.browser();
			}
		});
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
	
	module.audio = function(val, callback, opts) {
		var content = 	'<ul class = "nav nav-tabs" style="margin:auto -10px;padding:0 10px;">'+
							'<li class="active"><a href="#audio_browser" data-toggle="tab">网络音乐</a></li>'+
							'<li><a href="#audio_upload" data-toggle="tab">上传音乐</a></li>'+
						'</ul>'+
						'<div class = "tab-content form-horizontal" style="padding:20px 0;">'+
							'<div class="tab-pane active" id="audio_browser">'+
								'<div class="form-group">' +
									'<label class="col-xs-12 col-sm-2 control-label">音乐地址</label>' +
									'<div class="col-sm-10">' +
										'<div class="input-group">' +
											'<input class="form-control" type="text" id="audio_url" value="' + val + '" placeholder="请输入音乐URL"/>' +
											'<span class="input-group-btn">' +
												'<button class="btn btn-default btn-browser" type="button">浏览音乐空间</button>' +
											'</span>' +
										'</div>' +
									'</div>' +
								'</div>' +
							'</div>'+
							'<div class="tab-pane" id="audio_upload">'+
								'<iframe width="0" height="0" name="__audio_file_uploader" style="display:none;"></iframe>' +
								'<form action="./index.php?c=utility&a=file&do=upload&type=audio&callback=uploaderAudioComplete" enctype="multipart/form-data" method="post" target="__audio_file_uploader">'+
									'<div class="form-group">' +
										'<label class="col-xs-12 col-sm-2 control-label">上传音乐</label>' +
										'<div class="col-sm-10">' +
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
		var modalobj = module.dialog('请选择音乐', content, footer, {containerName: 'audio-container'}); 
		
		modalobj.modal({'keyboard': false});
		modalobj.find('button.btn-browser').off('click');
		modalobj.find('button.btn-browser').on('click', function(){
			
			var dialog = module.dialog('浏览音乐空间的音乐', '正在加载', footer, {containerName: 'audio-container'});
			dialog.find('.modal-dialog').css('width', '80%');
			dialog.find('.modal-dialog').css('min-height', 600);
			dialog.modal('show');
			
			window.audioBrowser = {
				attachpath : '',
				direct: function(val){
					var strs= val.split("/");
					dialog.find('.modal-body').load('./index.php?c=utility&a=file&do=browser&type=audio&callback=audioBrowser&path=' + '/'+strs[2]+'/'+strs[3], function(){
						var bread = dialog.find('.modal-body').find('.breadcrumb');
						bread.empty();
						bread.append('<li><a herf="javascript:;" onclick="audioBrowser.browser(\'/\');"><i class="fa fa-home">&nbsp;</i></a></li>');
						var str = '/'+strs[2];
						bread.append('<li><a herf="javascript:;" onclick="audioBrowser.browser(\''+str+'\');">'+strs[2]+'</a></li>');
						str = str +'/'+strs[3];
						bread.append('<li><a herf="javascript:;" onclick="audioBrowser.browser(\''+str+'\');">'+strs[3]+'</a></li>');
						var thumb = dialog.find('div.thumbnail[title="'+strs[4]+'"]');
						if(thumb.length>0){
							thumb.addClass('active')
						}
					});
				},
				browser: function(path){
					dialog.find('.modal-body').load('./index.php?c=utility&a=file&do=browser&type=audio&callback=audioBrowser&path=' + path, function(){
						var bread = dialog.find('.modal-body').find('.breadcrumb');
						bread.empty();
						var strs = [];
						if(path == '/'){
							bread.append('<li><a herf="javascript:;" onclick="audioBrowser.browser(\'/\');"><i class="fa fa-home">&nbsp;</i></a></li>');
						} else {
							strs= path.split("/");
							var str = '';
							bread.append('<li><a herf="javascript:;" onclick="audioBrowser.browser(\'/\');"><i class="fa fa-home"></i></a></li>');
							for(var i=1; i<strs.length; i++){
								str = str + '/'+strs[i];
								bread.append('<li><a herf="javascript:;" onclick="audioBrowser.browser(\''+str+'\');">'+strs[i]+'</a></li>');
							}
						}
					});
				},
				select: function(r) {
					callback({filename: r.filename, url: r.url});
					modalobj.modal('hide');
				},
				delete: function(file, path){
					$.get('./index.php?c=utility&a=file&do=delete&type=audio&file=' + file).success(function(dat){
						if(dat == 'success') {
							window.audioBrowser.browser(path);
						} else {
							var o = $.parseJSON(dat);
							module.message(o.message);
						}
					});
				}
			};
			window.audioBrowser.attachurl = (function(){
				var url=window.document.location.href; 
				var pathName = window.document.location.pathname; 
				var pos = url.indexOf(pathName); 
				var host = url.substring(0,pos);
				return host + '/attachment/';
			})();
			
			val = val.replace(window.audioBrowser.attachurl, '');
			val = val.replace(/(^\s*)|(\s*$)/g,"");
			var reg = /^audios\/[\d]+\/[\d]+\/[\d]+\/[\S]+/i;
			if(reg.test(val)){
				window.audioBrowser.direct(val);
			} else {
				window.audioBrowser.browser('/');
			}
		});
		modalobj.find('button.btn-primary').off('click');
		modalobj.find('button.btn-primary').on('click', function(){
			if(modalobj.find('.nav.nav-tabs li').eq(0).hasClass('active')) {
				var url = modalobj.find('#audio_url').val();
				callback({filename: url, url: url});
				modalobj.modal('hide');
			} else {
				modalobj.find(':hidden[name="options"]').val(opts);
				modalobj.find('form')[0].submit();
			}
		});
		require(['filestyle'], function($){
			modalobj.find(':file[name="file"]').filestyle({buttonText: '上传音乐'});
		});
		window.uploaderAudioComplete = function(r){
			if(r && r.filename && r.url) {
				callback({filename: r.filename, url: r.url});
				modalobj.modal('hide');
			} else {
				module.message(r.message,'','error');
			}
		};
	}; // end of audio
	/*
		打开远程地址
		@params string url 目标远程地址
		@params string title 打开窗口标题，为空则不显示标题。可在返回的HTML定义<button type="button" class="close" data-dismiss="modal" aria-hidden="true">×</button>控制关闭
		@params object options 打开窗口的属性配置，可选项backdrop,show,keyboard,remote,width,height。具体参考bootcss模态对话框的options说明
		@params object events 窗口的一些回调事件，可选项show,shown,hide,hidden,confirm。回调函数第一个参数对话框JQ对象。具体参考bootcss模态对话框的on说明.

		@demo ajaxshow('url', 'title', {'show' : true}, {'hidden' : function(obj) {obj.remove();}});
	*/
	module.ajaxshow = function(url, title, options, events) {

		var defaultoptions = {'show' : true};
		var defaultevents = {};
		var option = $.extend({}, defaultoptions, options);
		var events = $.extend({}, defaultevents, events);

		var footer = (typeof events['confirm'] == 'function' ? '<a href="#" class="btn btn-primary confirm">确定</a>' : '') + '<a href="#" class="btn" data-dismiss="modal" aria-hidden="true">关闭</a><iframe id="_formtarget" style="display:none;" name="_formtarget"></iframe>';
		var modalobj = module.dialog(title, '正在加载中', footer, {'containerName' : 'modal-panel-ajax'});

		if (typeof option['width'] != 'undeinfed' && option['width'] > 0) {
			modalobj.find('.modal-dialog').css({'width' : option['width']});
		}

		if (events) {
			for (i in events) {
				if (typeof events[i] == 'function') {
					modalobj.on(i, events[i]);
				}
			}
		}
		modalobj.find('.modal-body').load(url, function(){
			$('form').each(function(){
				$(this).attr('action', $(this).attr('action') + '&isajax=1');
				$(this).attr('target', '_formtarget');
			})
		});
		modalobj.on('hidden.bs.modal', function(){modalobj.remove();});
		if (typeof events['confirm'] == 'function') {
			modalobj.find('.confirm', modalobj).on('click', events['confirm']);
		}
		return modalobj.modal(option);
	}; //end of ajaxshow
	return module;
});
