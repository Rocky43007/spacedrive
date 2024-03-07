/* eslint-disable prettier/prettier */
// Requires Node.js 18+ to run.
import { JSDOM } from 'jsdom'

// type BrandData = {
// 	name: string
// 	url: string
// }

/**
 * Array of objects containing the brand name and URL
 * @type {Array<BrandData>}
 * @property {string} name - The name of the brand
 * @property {string} url - The URL of the brand
 * @property {string} maxPage - The maximum page number of the brand
 */
const brandsObj = []

/**
 *
 */
async function getBrandList() {
	const res = await fetch('https://www.gsmarena.com/makers.php3')

	if (!res.ok) {
		throw new Error('Failed to fetch brands')
	}

	// Get the HTML
	const html = await res.text()

	// Parse the HTML
	const dom = new JSDOM(html)
	/** @type {Document} */
	const document = dom.window.document
	const outerDiv = document.getElementsByClassName('st-text')[0]
	const brandListArray = Array.from(outerDiv.getElementsByTagName('a'))

	// Get the brand names and URLs
	brandListArray.forEach(brand => {
		let name = '';
		let isO2 = false;
		if (brand.textContent.trim().includes('O2')) {
			name = 'O2';
			isO2 = true;
		} else if (brand.textContent.trim().includes('Tel.Me.')) {
			name = 'Tel.Me.';
		} else {
			const match = brand.textContent.match(/[a-zA-Z-\s&]+/);
			name = match ? match[0] : '';
		}
		const url = brand.getAttribute('href');
		let maxPage = 0;
		if (!isO2) {
			const pageNumbers = brand.textContent.match(/\d+/g);
			if (pageNumbers) {
				maxPage = Math.ceil(Math.max(...pageNumbers.map(Number)) / 50);
			}
		} else {
			maxPage = 1;
		}
		if (name && url) {
			brandsObj.push({ name, url, maxPage });
		}
	});
}

await getBrandList()

const deviceURLs = {};

/**
 *
 * @param {{name: string, url: string, maxPage: number}} param0
 * @returns {Promise<void>}
 */
async function getBrandSpecificData({ name, url, maxPage }) {
	console.log('Fetching data for', name)
	const _url = url.split('-')
	for (let i = 0; i < maxPage; i++) {
		const pagedUrl = _url[0] + '-' + _url[1] + '-f-' + _url[2].split('.')[0] + '-0-' + `p${i}.php`
		const res = await fetch(`https://www.gsmarena.com/${pagedUrl}`)

		if (!res.ok) {
			throw new Error(`Failed to fetch ${name} data`)
		}

		// Get the HTML
		const html = await res.text()
		const dom = new JSDOM(html)
		/** @type {Document} */
		const document = dom.window.document
		const outerDiv = document.getElementsByClassName('makers')[0]
		const deviceListArray = Array.from(outerDiv.getElementsByTagName('a'))

		// Get the URLs
		deviceListArray.forEach(device => {
			const deviceURL = device.getAttribute('href')
			if (deviceURL) {
				if (deviceURLs[name]) {
					deviceURLs[name].push(deviceURL);
				} else {
					deviceURLs[name] = [deviceURL];
				}
			}
		})
	}

}

await getBrandSpecificData(brandsObj.filter(brand => brand.name === 'Samsung')[0])
await getBrandSpecificData(brandsObj.filter(brand => brand.name === 'BlackBerry')[0])
await getBrandSpecificData(brandsObj.filter(brand => brand.name === 'Celkon')[0])
await getBrandSpecificData(brandsObj.filter(brand => brand.name === 'Huawei')[0])
await getBrandSpecificData(brandsObj.filter(brand => brand.name === 'LG')[0])
await getBrandSpecificData(brandsObj.filter(brand => brand.name === 'Bird')[0])
