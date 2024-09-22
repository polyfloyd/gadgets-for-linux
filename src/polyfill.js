
let _style = document.createElement('style');
_style.type = 'text/css';
_style.innerHTML = `
	textarea[disabled] {
		display: none;
	}
`;
document.getElementsByTagName('head')[0].appendChild(_style);


class _Machine {
	get CPUs() {
		return [
			{ usagePercentage: Math.random() * 100 },
		];
	}

	get totalMemory() {
		return 100;
	}

	get availableMemory() {
		return Math.random() * 100;
	}
	get usagePercentage() {
		return Math.random() * 100;
	}
};

var System = {
	Gadget: {
		visible: true,
	},
	Machine: new _Machine(),
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
