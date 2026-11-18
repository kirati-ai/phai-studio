<?php
// For previewing with PHP's own server only (build.py --serve): what .htaccess does on Apache.
$path = parse_url($_SERVER['REQUEST_URI'], PHP_URL_PATH);
if (preg_match('#^/(storage|release\.json)#', $path)) {
    http_response_code(403);
    return true;
}
$file = $_SERVER['DOCUMENT_ROOT'] . $path;
if ($path !== '/' && (is_file($file) || is_file(rtrim($file, '/') . '/index.html'))) {
    return false;
}
require $_SERVER['DOCUMENT_ROOT'] . '/index.php';
