export function showStatus(statusContainer, message, type = 'info') {
    statusContainer.innerHTML = `<div class="status ${type}">${message}</div>`;

    if (type === 'success' || type === 'info') {
        setTimeout(() => {
            if (statusContainer) {
                statusContainer.innerHTML = '';
            }
        }, 3000);
    }
}

export function convertAnsiToHtml(ansiText) {
    if (!ansiText) return '';
    let html = ansiText
        .replace(/\u001b\[1m/g, '<strong>')
        .replace(/\u001b\[0m/g, '</span>')
        .replace(/\u001b\[32m/g, '<span style="color: #4CAF50;">')
        .replace(/\u001b\[33m/g, '<span style="color: #FFC107;">')
        .replace(/\u001b\[4;33m/g, '<span style="color: #FFC107; text-decoration: underline;">')
        .replace(/\u001b\[1;4;37m/g, '<span style="color: white; font-weight: bold; text-decoration: underline;">')
        .replace(/\u001b\[38;2;165;142;142m/g, '<span style="color: rgb(165,142,142);">')
        .replace(/\u001b\[48;2;50;50;50;37m/g, '<span style="background-color: rgb(50,50,50); color: white;">')
        .replace(/\u001b\[37m/g, '<span style="color: white;">')
        .replace(/\u001b\[1m/g, '<strong>')
        .replace(/\u001b\[0m/g, '</span>')
        .replace(/\u001b\[32m/g, '<span style="color: #4CAF50;">')
        .replace(/\u001b\[33m/g, '<span style="color: #FFC107;">')
        .replace(/\u001b\[4;33m/g, '<span style="color: #FFC107; text-decoration: underline;">')
        .replace(/\u001b\[1;4;37m/g, '<span style="color: white; font-weight: bold; text-decoration: underline;">')
        .replace(/\u001b\[38;2;165;142;142m/g, '<span style="color: rgb(165,142,142);">')
        .replace(/\u001b\[48;2;50;50;50;37m/g, '<span style="background-color: rgb(50,50,50); color: white;">')
        .replace(/\u001b\[37m/g, '<span style="color: white;">')
        .replace(/\n/g, '<br>');
    return html;
}

export function formatJsonAsYaml(obj, indent = 0) {
    const spaces = '  '.repeat(indent);
    let result = '';
    
    if (Array.isArray(obj)) {
        if (obj.length === 0) {
            return '[]';
        }
        for (let i = 0; i < obj.length; i++) {
            const item = obj[i];
            if (typeof item === 'object' && item !== null) {
                result += `${spaces}- \n${formatJsonAsYaml(item, indent + 1)}`;
            } else {
                result += `${spaces}- ${item}\n`;
            }
        }
    } else if (typeof obj === 'object' && obj !== null) {
        for (const [key, value] of Object.entries(obj)) {
            if (Array.isArray(value)) {
                if (value.length === 0) {
                    result += `${spaces}${key}: []\n`;
                } else if (value.every(v => typeof v !== 'object')) {
                    result += `${spaces}${key}: [${value.join(', ')}]\n`;
                } else {
                    result += `${spaces}${key}:\n${formatJsonAsYaml(value, indent + 1)}`;
                }
            } else if (typeof value === 'object' && value !== null) {
                result += `${spaces}${key}:\n${formatJsonAsYaml(value, indent + 1)}`;
            } else {
                result += `${spaces}${key}: ${value}\n`;
            }
        }
    } else {
        result += `${spaces}${obj}\n`;
    }
    
    return result;
}
