let global_MTX = -1;
let global_MTY = -1;

let tileNames = [];
tileNames[0] = 'EMPTY';
tileNames[1] = 'robot';
tileNames[2] = 'block';

let directionMap = [];
directionMap[0] = 'up';
directionMap[1] = 'right';
directionMap[2] = 'left';
directionMap[3] = 'down';

(function() {
	var resourceCache = {};
	var loading = [];
	var readyCallbacks = [];
	// Load an image url or an array of image urls
	function load(urlOrArr) {
		if(urlOrArr instanceof Array) {
			urlOrArr.forEach(function(url) {
				_load(url);
			}
		);
	}
        else {
            _load(urlOrArr);
        }
    }

    function _load(url) {
        if(resourceCache[url]) {
            return resourceCache[url];
        }
        else {
            var img = new Image();
            img.onload = function() {
			  console.log(url);
                resourceCache[url] = img;

                if(isReady()) {
                    readyCallbacks.forEach(function(func) { func(); });
                }
            };
            resourceCache[url] = false;
            img.src = url;
        }
    }

    function get(url) {
        return resourceCache[url];
    }

    function isReady() {
        var ready = true;
        for(var k in resourceCache) {
            if(resourceCache.hasOwnProperty(k) &&
               !resourceCache[k]) {
                ready = false;
            }
        }
        return ready;
    }

    function onReady(func) {
        readyCallbacks.push(func);
    }

    window.resources = {
        load: load,
        get: get,
        onReady: onReady,
        isReady: isReady
    };
})();

class Tile {
	constructor(id, orient) {
		this.id = id;
		this.orientation = orient;
	}
}

class MapDisp {
	constructor(width, height) {
		this._height = height;
		this._width = width;
		this.map = [];
		resources.load(['/res/grid.png', '/res/mouse_over.png', '/res/tile1_0.png', '/res/tile1_1.png', '/res/tile1_2.png', '/res/tile1_3.png', '/res/tile2_0.png']);
	}

	setLine(x, line) {
		this.map[x] = line;
	}

	initializeMap() {
		for (var x = 0; x < this._width; x++) {
			let addLine = [];
			for (var y = 0; y < this._height; y++) {
				let addTile = new Tile(0, 0);
				addLine[y] = addTile;
			}
			this.setLine(x, addLine);
		}
	}

	makeMap() {
		for (var x = 0; x < this._width; x++) {
			let addLine = [];
			for (var y = 0; y < this._height; y++) {
				let id = 0;
				let orient = 0;

				if (x > 10 && y > 6) {
					id = 2;
				} else if (x === 5 && y === 2) {
					id = 1;
					orient = 2;
				} else if (x===2 && y === 5) {
					id = 1;
					orient = 0;
				}

				let addTile = new Tile(id, orient);
				addLine[y] = addTile;
			}
			this.setLine(x, addLine);
		}
	}
}

currentMap = new MapDisp(canvas.width/32, canvas.height/32);
currentMap.makeMap();

function renderMap() {
	if (resources.isReady()) {
		for (var x = 0; x < canvas.width/32; x++) {
			for (var y = 0; y < canvas.height/32; y++) {
				let id = currentMap.map[x][y].id;
				let orient = currentMap.map[x][y].orientation;
				let gridSpr = resources.get('/res/grid.png');
				ctx.drawImage(gridSpr, x*32, y*32, 32, 32);
				if (id != 0) {
					let tileSpr = resources.get('/res/tile' + id.toString() + '_' + orient.toString() + '.png');
					ctx.drawImage(tileSpr, x*32, y*32, 32, 32);
				}
			}
		}
	}
}

function renderCursor() {
	if (resources.isReady()) {
		let mSpr = resources.get('/res/mouse_over.png');
		mtx.fillStyle = "rgba(255, 255, 255, 0.0)";
		mtx.clearRect(0, 0, canvas.width, canvas.height);
		mtx.drawImage(mSpr, global_MTX*32, global_MTY*32, 33, 33);
	}
}

function setStrFixed(strStr, maxLen) {
	if (strStr.length > maxLen) {
		return strStr.substr(0, maxLen-3) + "...";
	} else if (strStr.length < maxLen) {
		neededChars = maxLen - strStr.length;
		return '&nbsp;'.repeat(neededChars) + strStr;
	} else {
		return strStr;
	}
}

function setMapData() {
	let dmx = global_MTX.toString();
	let dmy = global_MTY.toString();
	let tid = currentMap.map[global_MTX][global_MTY].id.toString();
	let htmlMake = 'Mouse X: <span class="ctext-const">' + setStrFixed(dmx, 6) + '</span> | ';
	htmlMake += '    Mouse Y: <span class="ctext-const">' + setStrFixed(dmy, 6) + '</span> | ';
	htmlMake += '    Tile Id: <span class="ctext-reg">' + setStrFixed(tid, 4) + '</span> | ';
	document.getElementById("map_data").innerHTML = htmlMake;
}

canvas.addEventListener('mousemove', function(evt) {
	var mousePos = getMousePos(canvas, evt);

	if (mousePos.x >= 0 && mousePos.y >= 0) {
		global_MTX = Math.floor(mousePos.x/32);
		global_MTY = Math.floor(mousePos.y/32);
		renderCursor();
		setMapData();
	}
}, false);

canvas.addEventListener('mouseleave', function(evt) {
	mtx.fillStyle = "rgba(255, 255, 255, 0.0)";
	mtx.clearRect(0, 0, canvas.width, canvas.height);
	document.getElementById("map_data").innerHTML = '';
}, false)
