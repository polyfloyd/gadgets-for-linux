
let _style = document.createElement('style');
_style.type = 'text/css';
_style.innerHTML = `
	textarea[disabled] {
		display: none;
	}
`;
document.getElementsByTagName('head')[0].appendChild(_style);


class Settings {
	constructor() {
		this.values = {};
	}

	read(key) {
		return this._values || null;
	}

	readString(key) {
		return this.read(key) || '';
	}

	write(key, value) {
		this.values[key] = value;
	}
}

var System = {
	Gadget: {
		visible: true,
		path: 'dummy.gadget',
		Settings: new Settings(),
	},
	Machine: null, // Set separately.
};


Object.defineProperty(Array.prototype, 'count', {
	get() { return this.length; },
});

Array.prototype.item = function(i) {
	return this[i];
}


// https://learn.microsoft.com/en-us/previous-versions/windows/desktop/sidebar/image-element
Object.defineProperty(HTMLImageElement.prototype, 'Rotation', {
	set(r) {
		this.style.transform = `rotate(${r}deg)`;
	},
});

Object.defineProperty(HTMLImageElement.prototype, 'src', {
	set(s) {
		s = s.replace(/url\((.+)\)/, '$1');
		this.setAttribute('src', s);
	},
});

// https://learn.microsoft.com/en-us/previous-versions/windows/desktop/sidebar/addshadow-method-gimage
HTMLImageElement.prototype.addShadow = function(color, radius, alpha, dx, dy) {
	this.style.filter = `drop-shadow(${dx}px ${dy}px ${radius}px ${color})`;
}

// https://learn.microsoft.com/en-us/previous-versions/windows/desktop/sidebar/addglow-method-gimage
HTMLImageElement.prototype.addGlow = function(color, radius, alpha) {
	this.style.filter = `drop-shadow(0 0 ${radius}px ${color})`;
}


class ActiveXObject {
	constructor() {
		this.values = {};
	}

	RegRead(path) {
		return this._values || 'default';
	}

	RegWrite(path, value) {
		this.values[path] = value;
	}

	FileExists(path) {
		return false;
	}
}
