const puppeteer = require('puppeteer-core');
const fs = require('fs');

(async () => {
    // Find Chrome
    const executablePath = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
    
    console.log('Launching browser...');
    const browser = await puppeteer.launch({
        executablePath,
        headless: 'new',
        args: ['--no-sandbox', '--window-size=1280,1080']
    });

    const page = await browser.newPage();
    await page.setViewport({ width: 1280, height: 1080 });
    
    console.log('Navigating to youtube...');
    await page.goto('https://www.youtube.com', { waitUntil: 'domcontentloaded' });
    console.log('Waiting 3s...');
    await new Promise(r => setTimeout(r, 3000));
    
    console.log('Extracting DOM...');
    const script = fs.readFileSync('src/dom.rs', 'utf8').split('r#"')[1].split('"#')[0];
    
    try {
        const jsonStr = await page.evaluate(script);
        const data = JSON.parse(jsonStr);
        console.log(`DOM extracted. Root elements: ${data.e.length}`);
        if (data.e.length < 5) console.log(data);
    } catch (e) {
        console.error('Extraction failed:', e);
    }

    await browser.close();
})();
