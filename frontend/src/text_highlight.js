var $backdrop = $('.backdrop');
var $backdrop_text = $('.backdrop_text');
var $highlights = $('.highlights');
var $highlights_text = $('.highlights_text')
var textarea = document.getElementById("editor");

var ua = window.navigator.userAgent.toLowerCase();
var isIE = !!ua.match(/msie|trident\/7|edge/);
var isWinPhone = ua.indexOf('windows phone') !== -1;
var isIOS = !isWinPhone && !!ua.match(/ipad|iphone|ipod/);

function pasteHtmlAtCaret(html) {
	var sel, range;
	if (window.getSelection) {
		// IE9 and non-IE
		sel = window.getSelection();
		if (sel.getRangeAt && sel.rangeCount) {
			range = sel.getRangeAt(0);
			console.log(range);
			range.deleteContents();
			// Range.createContextualFragment() would be useful here but is
			// non-standard and not supported in all browsers (IE9, for one)
               var el = document.createElement("div");
               el.innerHTML = html;
               var frag = document.createDocumentFragment(), node, lastNode;
               while ( (node = el.firstChild) ) {
                    lastNode = frag.appendChild(node);
               }
               range.insertNode(frag);
               // Preserve the selection
               if (lastNode) {
                    range = range.cloneRange();
                    range.setStartAfter(lastNode);
                    range.collapse(true);
                    sel.removeAllRanges();
                    sel.addRange(range);
               }
          }
     } else if (document.selection && document.selection.type != "Control") {
          // IE < 9
          document.selection.createRange().pasteHTML(html);
     }
}

function applyHighlights(text) {
	text = text.replace(/\n/g, '<br>');
	text = text.replace(/%[a-z]{1,3}\b/g, '<mark-reg>$&</mark-reg>')
	text = text.replace(/\b[0-9]{1,3}\b/g, '<mark-const>$&</mark-const>');
	text = text.replace(/\$[a-z|_][a-z|0-9|_]*/g, '<mark-var>$&</mark-var>');
	text = text.replace(/[a-z|_][a-z|0-9|_]*:/g, '<mark-label>$&</mark-label>');
//	text = text.replace(/\/\/[a-z| |A-Z|0-9]*/g, '<mark-comment>$&</mark-comment>');

	text = text.replace(/%[a-z]{1,3}\b/g, '<span class="ctext-reg">$&</span>')
	text = text.replace(/\b[0-9]{1,3}\b/g, '<span class="ctext-const">$&</span>');
	text = text.replace(/\$[a-z|_][a-z|0-9|_]*/g, '<span class="ctext-var">$&</span>');
	text = text.replace(/[a-z|_][a-z|0-9|_]*:/g, '<span class="ctext-label">$&</span>');
	text = text.replace(/\/\/[a-z| |A-Z|0-9|&nbsp;]*/g, '<span class="ctext-comment">$&</span>');

	if (isIE) {
		// IE wraps whitespace differently in a div vs textarea, this fixes it
		text = text.replace(/ /g, ' <wbr>');
	}
	return text;
}

function handleInput(event) {
	if (event == undefined) {
		return;
	}

	var text = event.target.innerHTML;


	var highlightedText = applyHighlights(text);
	$highlights.html(highlightedText);
	$highlights_text.html(highlightedText);
}

function handleScroll(event) {
	var thisBackdrop = document.getElementById("editor_backdrop");
	var textBackdrop = document.getElementById("editor_text");
	var scrollTop = textarea.scrollTop;
	thisBackdrop.scrollTop = scrollTop;
	textBackdrop.scrollTop = scrollTop;

	var scrollLeft = textarea.scrollLeft;
	thisBackdrop.scrollLeft = scrollLeft;
	textBackdrop.scrollLeft = scrollLeft;
}

function fixIOS() {
	// iOS adds 3px of (unremovable) padding to the left and right of a textarea, so adjust highlights div to match
	$highlights.css({
		'padding-left': '+=3px',
		'padding-right': '+=3px'
	});
}

function bindEvents() {
	textarea.addEventListener('input', handleInput);
	textarea.addEventListener('scroll', handleScroll);
}

if (isIOS) {
	fixIOS();
}

bindEvents();
handleInput();
