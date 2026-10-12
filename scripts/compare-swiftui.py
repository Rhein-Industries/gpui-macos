#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy>=2", "pillow>=11", "scikit-image>=0.25", "pyobjc-framework-Quartz>=11", "pyobjc-framework-Cocoa>=11"]
# ///
"""Capture PID-owned macOS gallery windows and compare against SwiftUI.

uv run scripts/compare-swiftui.py --skip-build
Regions: --regions path.json, in window points. `default` {x,y,right,bottom}
is the `content` region; `regions` {name: {x,y,right|width,bottom|height}} adds
named regions measured from the same capture (for example `sidebar`); `pages`
{slug: {x,y,..., regions: {name: {...}}}} overrides either per page.
No image resizing or registration.
Threshold is the pixelmatch YIQ tolerance (0..1); antialias pixels are included.
Category 3 pairs have diagnostics but are excluded from rankings.
"""
from __future__ import annotations

import argparse
import datetime as dt
import html
import json
import math
import os
from pathlib import Path
import plistlib
import re
import signal
import subprocess
import sys
import time

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
PILOT = ['checkbox', 'switch', 'button', 'select', 'table']


def command(args):
    return subprocess.check_output(args, text=True, stderr=subprocess.PIPE).strip()


def preference(domain, key):
    result = subprocess.run(['defaults', 'read', domain, key], capture_output=True, text=True)
    if result.returncode == 0:
        return result.stdout.strip()
    # A missing key is expected; do not silently treat other failures as absence.
    if 'does not exist' in result.stderr or 'Could not find' in result.stderr:
        return None
    raise RuntimeError(f"Cannot read {domain}/{key}: {result.stderr.strip()}")


def inventory():
    """Page slug -> {name, category, group}; category 1 = system equivalent,
    2 = composed design, 3 = no native equivalent (excluded from ranking)."""
    return json.loads((ROOT / 'scripts/compare-swiftui/pages.json').read_text())


WINDOW_SIZE = (1060, 760)


def active_displays():
    import Quartz as Q
    error, ids, count = Q.CGGetActiveDisplayList(32, None, None)
    if error:
        raise RuntimeError(f'Cannot enumerate active displays: CGError {error}')
    if not count:
        raise RuntimeError('No active displays visible; WindowServer access requires an interactive desktop session')
    import AppKit
    screens = {int(screen.deviceDescription()['NSScreenNumber']): screen for screen in AppKit.NSScreen.screens()}
    displays = []
    for display_id in ids[:count]:
        screen = screens.get(int(display_id))
        if screen is None:
            raise RuntimeError(f'Active display {display_id} has no NSScreen')
        bounds = Q.CGDisplayBounds(display_id)
        displays.append({'id': int(display_id), 'name': str(screen.localizedName()),
                         'scale': float(screen.backingScaleFactor()),
                         'width_points': bounds.size.width, 'height_points': bounds.size.height,
                         'bounds_points': {'X': bounds.origin.x, 'Y': bounds.origin.y,
                                           'Width': bounds.size.width, 'Height': bounds.size.height}})
    return displays


def select_display(displays, requested, main_id):
    if not displays:
        raise RuntimeError('No active displays')
    if requested == 'auto':
        # Stable tie-break: prefer the main screen among equal backing scales.
        return max(displays, key=lambda d: (d['scale'], d['id'] == main_id))
    try:
        display_id = main_id if requested == 'main' else int(requested)
    except ValueError:
        raise RuntimeError('--display must be auto, main, or an active CGDirectDisplayID') from None
    for display in displays:
        if display['id'] == display_id:
            return display
    raise RuntimeError(f'Requested display {display_id} is not active')


def validate_window(bounds, display):
    size = (bounds['Width'], bounds['Height'])
    if size != WINDOW_SIZE:
        raise RuntimeError(f'Window frame is {size[0]}x{size[1]} pt; expected 1060x760 pt including titlebar')
    screen = display['bounds_points']
    if not (bounds['X'] >= screen['X'] and bounds['Y'] >= screen['Y']
            and bounds['X'] + bounds['Width'] <= screen['X'] + screen['Width']
            and bounds['Y'] + bounds['Height'] <= screen['Y'] + screen['Height']):
        raise RuntimeError(f"Window frame {bounds} is not wholly on display {display['name']} ({display['id']})")


def validate_capture(record, display):
    expected = [int(point * display['scale']) for point in WINDOW_SIZE]
    if record['size_pixels'] != expected or any(not math.isclose(scale, display['scale'], abs_tol=1e-6) for scale in record['capture_scale']):
        raise RuntimeError(f"Capture is {record['size_pixels']} px at {record['capture_scale']} scale; expected {expected} px at {display['scale']}x on {display['name']} ({display['id']})")


def validate_pair(captures):
    records = list(captures.values())
    for field in ['size_pixels', 'capture_scale']:
        if any(record[field] != records[0][field] for record in records[1:]):
            raise RuntimeError(f'Cannot compare mismatched {field}: ' + str({app: record[field] for app, record in captures.items()}))
    sizes = [(r['window_bounds_points']['Width'], r['window_bounds_points']['Height']) for r in records if 'window_bounds_points' in r]
    if sizes and any(size != sizes[0] for size in sizes[1:]):
        raise RuntimeError(f'Cannot compare mismatched window point sizes: {sizes}')


def preflight(requested_display='auto', bounds_only=False):
    if sys.platform != 'darwin':
        raise RuntimeError('Window capture requires macOS and an interactive desktop session.')
    import Quartz as Q
    accent = {key: preference('-g', key) for key in ['AppleAccentColor', 'AppleHighlightColor']}
    if any(value is not None for value in accent.values()):
        raise RuntimeError('Set the host accent to default blue: AppleAccentColor and AppleHighlightColor must both be absent. Current values: ' + json.dumps(accent))
    if not bounds_only and not Q.CGPreflightScreenCaptureAccess():
        raise RuntimeError('Screen Recording permission is missing. Enable Screen & System Audio Recording for the terminal/runner in System Settings, restart it, and retry. No captures were launched.')
    accessibility = {}
    for key in ['reduceTransparency', 'increaseContrast']:
        raw = preference('com.apple.universalaccess', key)
        accessibility[key] = {'raw': raw, 'enabled': raw not in (None, '0', 'false', 'NO')}
    displays = active_displays()
    display = select_display(displays, requested_display, int(Q.CGMainDisplayID()))
    return {
        'timestamp_utc': dt.datetime.now(dt.timezone.utc).isoformat(),
        'macos': command(['sw_vers']),
        'xcode': command(['xcodebuild', '-version']),
        'sdk': command(['xcrun', '--sdk', 'macosx', '--show-sdk-version']),
        'display': display,
        'active_displays': displays,
        'accent': accent,
        'accessibility': accessibility,
        'capture': 'CGWindowList bounds only; no pixels captured' if bounds_only else 'screencapture -x -o -l; active fresh launch; no shadow; largest on-screen layer-0 PID window',
    }


def executable(bundle, fallback=None):
    info = bundle / 'Contents/Info.plist'
    if info.exists():
        with info.open('rb') as stream:
            path = bundle / 'Contents/MacOS' / plistlib.load(stream)['CFBundleExecutable']
        if path.is_file() and os.access(path, os.X_OK):
            return path
    if fallback and fallback.is_file() and os.access(fallback, os.X_OK):
        return fallback
    raise RuntimeError(f'App executable missing: {bundle}. Build it first or omit --skip-build.')


def main_window(pid):
    import Quartz as Q
    windows = Q.CGWindowListCopyWindowInfo(Q.kCGWindowListOptionOnScreenOnly | Q.kCGWindowListExcludeDesktopElements, Q.kCGNullWindowID) or []
    candidates = [w for w in windows if w.get(Q.kCGWindowOwnerPID) == pid and w.get(Q.kCGWindowLayer) == 0 and w.get(Q.kCGWindowAlpha, 1) > 0 and w[Q.kCGWindowBounds]['Width'] > 1 and w[Q.kCGWindowBounds]['Height'] > 1]
    return max(candidates, key=lambda w: w[Q.kCGWindowBounds]['Width'] * w[Q.kCGWindowBounds]['Height'], default=None)


def frontmost_application():
    from AppKit import NSWorkspace
    from Foundation import NSDate, NSRunLoop
    # NSWorkspace learns about activation through run-loop notifications; without
    # pumping, a script keeps reading the value cached at its first query.
    NSRunLoop.currentRunLoop().runUntilDate_(NSDate.dateWithTimeIntervalSinceNow_(0.01))
    return NSWorkspace.sharedWorkspace().frontmostApplication()


def frontmost_pid():
    app = frontmost_application()
    return app.processIdentifier() if app else None


def is_frontmost(pid):
    return frontmost_pid() == pid


def restore_focus(previous, record):
    """Hand keyboard focus back to the app the user was in before the capture."""
    if previous is None or previous.isTerminated():
        return
    previous.activateWithOptions_(0)
    deadline = time.monotonic() + 1
    while time.monotonic() < deadline and frontmost_pid() != previous.processIdentifier():
        time.sleep(.05)
    if frontmost_pid() != previous.processIdentifier() and previous.bundleIdentifier():
        # Cooperative activation can be refused for a background caller; Launch Services is not.
        subprocess.run(['open', '-b', previous.bundleIdentifier()], capture_output=True, timeout=5)
    record['focus_restored_to'] = previous.localizedName()


def hid_idle_seconds():
    output = command(['ioreg', '-c', 'IOHIDSystem'])
    match = re.search(r'"HIDIdleTime"\s*=\s*(\d+)', output)
    if not match:
        raise RuntimeError('ioreg did not report HIDIdleTime; cannot safely wait for idle')
    return int(match[1]) / 1_000_000_000


def wait_for_idle(seconds):
    if seconds <= 0:
        return
    print(f'Waiting for user idle >= {seconds:g}s (HIDIdleTime) before taking focus', flush=True)
    while hid_idle_seconds() < seconds:
        time.sleep(.5)


class CaptureInterruptedByInput(Exception):
    pass


def settle_capture(seconds, wait_idle):
    if not wait_idle:
        time.sleep(seconds)
        return
    deadline = time.monotonic() + seconds
    while True:
        if hid_idle_seconds() < 5:
            raise CaptureInterruptedByInput()
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return
        time.sleep(min(.1, remaining))


def capture(path, page, appearance, destination, settle, timeout, record, display, bounds_only=False, wait_idle=0, extra_arguments=()):
    while True:
        wait_for_idle(wait_idle)
        record.clear()
        try:
            return capture_once(path, page, appearance, destination, settle, timeout, record, display, bounds_only, wait_idle, extra_arguments)
        except CaptureInterruptedByInput:
            destination.unlink(missing_ok=True)
            print('User input during settling; discarded capture and waiting again', flush=True)


def capture_once(path, page, appearance, destination, settle, timeout, record, display, bounds_only=False, wait_idle=0, extra_arguments=()):
    import Quartz as Q
    started = time.monotonic()
    previous = frontmost_application()
    hidden = False
    process = None
    arguments = [page, '--' + appearance, '--display', str(display['id']), *extra_arguments]
    record.update({'executable': str(path), 'arguments': arguments, 'display': display})
    # Log output beside the image; pipe buffers must never block app startup.
    with destination.with_suffix('.log').open('wb') as log:
        try:
            process = subprocess.Popen([str(path), *arguments], cwd=ROOT, stdout=log, stderr=log)
            record['pid'] = process.pid
            deadline = started + timeout
            while True:
                if process.poll() is not None:
                    raise RuntimeError(f'{path.name} exited with {process.returncode}; see {destination.with_suffix(".log")}')
                window = main_window(process.pid)
                # Active look: wait until the app owns the key window, not just a window.
                if window and is_frontmost(process.pid):
                    break
                if time.monotonic() > deadline:
                    raise RuntimeError(f'No active on-screen layer-0 window for PID {process.pid} within {timeout}s')
                time.sleep(.05)
            # Popups present after launch and animate in; give them extra time.
            settle_capture(settle + (1.5 if '--open' in extra_arguments else 0), wait_idle)
            window = main_window(process.pid)
            if process.poll() is not None or not window:
                raise RuntimeError(f'PID {process.pid} lost its window before capture')
            if not is_frontmost(process.pid):
                raise RuntimeError(f'PID {process.pid} lost focus before capture; its window would render inactive')
            record['active'] = True
            record['window_id'] = int(window[Q.kCGWindowNumber])
            record['window_bounds_points'] = dict(window[Q.kCGWindowBounds])
            record['window_title'] = window.get(Q.kCGWindowName)
            validate_window(record['window_bounds_points'], display)
            record['display_backing_scale'] = display['scale']
            if bounds_only:
                return record
            try:
                hidden = Q.CGDisplayHideCursor(display['id']) == 0
            except Exception:
                pass
            if '--open' in extra_arguments:
                # Popups, menus, sheets and alerts are separate windows; capture the
                # screen area of the main window so they are included.
                b = record['window_bounds_points']
                rect = f"{b['X']:g},{b['Y']:g},{b['Width']:g},{b['Height']:g}"
                record['capture_method'] = 'screen-rect'
                subprocess.run(['screencapture', '-x', '-R', rect, str(destination)], check=True, capture_output=True, timeout=timeout)
            else:
                record['capture_method'] = 'window'
                subprocess.run(['screencapture', '-x', '-o', '-l', str(record['window_id']), str(destination)], check=True, capture_output=True, timeout=timeout)
            with Image.open(destination) as image:
                image.load()
                record['size_pixels'] = list(image.size)
            bounds = record['window_bounds_points']
            record['capture_scale'] = [record['size_pixels'][0] / bounds['Width'], record['size_pixels'][1] / bounds['Height']]
            validate_capture(record, display)
            record['cursor_hidden'] = hidden
            return record
        finally:
            if hidden:
                try:
                    Q.CGDisplayShowCursor(display['id'])
                except Exception as error:
                    record['cursor_restore_error'] = str(error)
            try:
                if process is not None:
                    if process.poll() is None:
                        process.terminate()  # Only the retained Popen PID, never a name/group.
                        try:
                            process.wait(timeout=5)
                        except subprocess.TimeoutExpired:
                            process.kill()
                            process.wait(timeout=5)
                    record['exit_code_after_cleanup'] = process.returncode
                    record['process_reaped'] = process.poll() is not None
            finally:
                restore_focus(previous, record)


def rgb(image):
    # Composite transparent rounded corners consistently onto white.
    image = image.convert('RGBA')
    background = Image.new('RGBA', image.size, 'white')
    return np.asarray(Image.alpha_composite(background, image).convert('RGB'))


def color_delta(a, b):
    delta = a.astype(np.float64) - b.astype(np.float64)
    y = delta @ np.array([.29889531, .58662247, .11448223])
    i = delta @ np.array([.59597799, -.27417610, -.32180189])
    q = delta @ np.array([.21147017, -.52261711, .31114694])
    return .5053 * y*y + .299 * i*i + .1957 * q*q


def metrics(a, b, delta, threshold):
    from skimage.metrics import structural_similarity
    height, width = a.shape[:2]
    count = int(np.count_nonzero(delta > 35215 * threshold**2))
    win = min(7, height, width)
    if win % 2 == 0:
        win -= 1
    ssim = float(structural_similarity(a, b, channel_axis=2, data_range=255, win_size=win)) if win >= 3 else None
    return {'size_pixels': [width, height], 'pixel_count': width*height, 'mismatched_pixels': count, 'mismatch_percent': count / (width*height) * 100, 'ssim': ssim}


REGION_KEYS = {'x', 'y', 'right', 'bottom', 'width', 'height'}


def page_regions(config, page):
    """Named regions for one page: `content` (default plus page keys, as before)
    and every configured named region, each merged with its page override."""
    page_config = config.get('pages', {}).get(page, {})
    page_named = page_config.get('regions', {})
    regions = {'content': {**config['default'], **{k: v for k, v in page_config.items() if k != 'regions'}}}
    for name in [*config.get('regions', {}), *page_named]:
        if name != 'content':
            regions[name] = {**config.get('regions', {}).get(name, {}), **page_named.get(name, {})}
    for name, region in regions.items():
        if not {'x', 'y'} <= region.keys() or region.keys() - REGION_KEYS:
            raise RuntimeError(f'Region {name!r} needs x and y and only {sorted(REGION_KEYS)}')
        if any(not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0 for value in region.values()):
            raise RuntimeError('Region values must be finite nonnegative point offsets')
    return regions


def region_box(region, captures, width, height):
    """Pixel box [x, y, right, bottom] common to both captures. `width` /
    `height` give an extent from x / y; otherwise `right` / `bottom` are insets."""
    boxes = []
    for capture_info in captures.values():
        sx, sy = capture_info['capture_scale']
        pw, ph = capture_info['size_pixels']
        right = math.ceil((region['x'] + region['width'])*sx) if 'width' in region else pw-math.ceil(region.get('right', 0)*sx)
        bottom = math.ceil((region['y'] + region['height'])*sy) if 'height' in region else ph-math.ceil(region.get('bottom', 0)*sy)
        boxes.append([math.ceil(region['x']*sx), math.ceil(region['y']*sy), right, bottom])
    return [max(box[0] for box in boxes), max(box[1] for box in boxes), min(width, *(box[2] for box in boxes)), min(height, *(box[3] for box in boxes))]


def compare(folder, captures, regions, threshold):
    """Full-window and per-region metrics. `regions` maps names to point
    regions; a single region (with `x`) is the `content` region."""
    if 'x' in regions:
        regions = {'content': regions}
    validate_pair(captures)
    with Image.open(folder / 'gpui.png') as ga, Image.open(folder / 'swiftui.png') as sw:
        a, b = rgb(ga), rgb(sw)
    if a.shape != b.shape:
        raise RuntimeError(f'Cannot compare mismatched image dimensions: {a.shape} / {b.shape}')
    height, width = a.shape[:2]
    delta = color_delta(a, b)
    # Region offsets are points; convert to capture pixels (2x on Retina).
    named = {}
    for name, region in regions.items():
        x, y, right, bottom = region_box(region, captures, width, height)
        if right <= x or bottom <= y:
            if name == 'content':
                raise RuntimeError(f'Content region is empty on common canvas: {[x,y,right,bottom]}')
            named[name] = {'error': f'Region is empty on common canvas: {[x,y,right,bottom]}', 'region_points': region}
            continue
        named[name] = {**metrics(a[y:bottom, x:right], b[y:bottom, x:right], delta[y:bottom, x:right], threshold), 'box_pixels': [x,y,right,bottom], 'region_points': region}
    full = metrics(a, b, delta, threshold)
    content = {k: v for k, v in named['content'].items() if k != 'region_points'}
    strength = np.clip(np.sqrt(delta / 35215), 0, 1)
    heat = np.zeros_like(a)
    heat[..., 0] = np.where(delta > 35215*threshold**2, 255, 0)
    heat[..., 1] = (strength*180).astype(np.uint8)
    Image.fromarray(heat).save(folder / 'heatmap.png')
    Image.blend(Image.fromarray(a), Image.fromarray(b), .5).save(folder / 'overlay.png')
    Image.fromarray(np.concatenate([a, b], axis=1)).save(folder / 'side-by-side.png')
    # `content` and `region_points` keep their pre-region shape for baselines.
    return {'sizes_differ': captures['gpui']['size_pixels'] != captures['swiftui']['size_pixels'], 'scales_differ': captures['gpui']['capture_scale'] != captures['swiftui']['capture_scale'], 'comparison_canvas': 'identical size and scale, no resizing', 'full': full, 'content': content, 'region_points': regions['content'], 'regions': named}


def validate_output(out, overwrite=False):
    if out.exists() and (not out.is_dir() or (any(out.iterdir()) and not overwrite)):
        raise RuntimeError(f'Output directory is non-empty or not a directory: {out}; use --overwrite to reuse a non-empty directory')


def metric_delta(current, prior):
    if not current or not prior or 'mismatch_percent' not in current or 'mismatch_percent' not in prior:
        return None
    return {
        'mismatch_percent_points': current['mismatch_percent'] - prior['mismatch_percent'],
        'ssim': None if current['ssim'] is None or prior['ssim'] is None else current['ssim'] - prior['ssim'],
    }


def baseline_deltas(results, baseline):
    previous = {(p['page'], p['appearance']): p for p in baseline['pairs']}
    for pair in results['pairs']:
        old = previous.get((pair['page'], pair['appearance']))
        delta = metric_delta(pair.get('metrics', {}).get('content'), (old or {}).get('metrics', {}).get('content'))
        if delta is None:
            pair['baseline_delta'] = {'status': 'unavailable'}
            continue
        pair['baseline_delta'] = {
            'status': 'compared',
            'content_mismatch_percent_points': delta['mismatch_percent_points'],
            'content_ssim': delta['ssim'],
        }
        # Named regions other than content, when both runs measured them.
        current_regions = pair.get('metrics', {}).get('regions', {})
        prior_regions = (old or {}).get('metrics', {}).get('regions', {})
        regions = {name: region for name in current_regions if name != 'content'
                   for region in [metric_delta(current_regions[name], prior_regions.get(name))] if region is not None}
        if regions:
            pair['baseline_delta']['regions'] = regions


def print_baseline_deltas(results):
    for pair in results['pairs']:
        delta = pair['baseline_delta']
        prefix = f"Baseline delta {pair['page']} / {pair['appearance']}: "
        if delta['status'] != 'compared':
            print(prefix + 'unavailable (missing content metrics)', flush=True)
        else:
            ssim = 'unavailable' if delta['content_ssim'] is None else f"{delta['content_ssim']:+.5f}"
            print(prefix + f"content mismatch {delta['content_mismatch_percent_points']:+.3f} percentage points; SSIM {ssim}", flush=True)
            for name, region in delta.get('regions', {}).items():
                ssim = 'unavailable' if region['ssim'] is None else f"{region['ssim']:+.5f}"
                print(prefix + f"{name} mismatch {region['mismatch_percent_points']:+.3f} percentage points; SSIM {ssim}", flush=True)


def write_report(out, results):
    (out / 'results.json').write_text(json.dumps(results, indent=2, allow_nan=False) + '\n')
    data = json.dumps(results, allow_nan=False).replace('<', '\\u003c')
    template = '''<!doctype html><html lang="en"><meta charset="utf-8"><title>GPUI / SwiftUI comparison</title>
<style>body{font:14px system-ui;margin:24px;background:#eee;color:#222}button{margin:4px;padding:8px}article{background:white;padding:16px;margin:16px 0}img{max-width:100%}.stack{position:relative;display:inline-block;max-width:100%}.stack img{display:block}.stack{overflow:hidden}.stack img{position:absolute;top:0;left:0;max-width:none}.stack .over{position:absolute;top:0;left:0}pre{white-space:pre-wrap}a{margin-right:12px}.excluded{opacity:.7}</style>
<h1>GPUI / SwiftUI comparison</h1><p>Ranked by content mismatch (highest first) within inventory category. Category 1: system equivalent; 2: composed design; 3: placeholder, excluded from ranking. Full-window scores include desktop-dependent glass. Frames and backing scales must match; no resizing.</p>
<nav><button onclick="render('light')">Light</button><button onclick="render('dark')">Dark</button></nav>
<label>SwiftUI overlay opacity <input id="opacity" type="range" min="0" max="100" value="50" oninput="document.querySelectorAll('.over').forEach(e=>e.style.opacity=this.value/100)"> </label>
<main id="report"></main><details><summary>Manifest</summary><pre id="manifest"></pre></details>
<script>const data=__DATA__;const esc=s=>String(s).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const pct=n=>n.toFixed(3)+'%';function render(appearance){let output='';for(const category of [1,2,3]){let rows=data.pairs.filter(p=>p.appearance===appearance&&p.category===category);if(!rows.length)continue;rows.sort((a,b)=>category===3?a.page.localeCompare(b.page):(b.metrics?.content.mismatch_percent??-1)-(a.metrics?.content.mismatch_percent??-1));output+='<h2>Category '+category+'</h2>';for(const p of rows){const base=encodeURIComponent(p.page)+'/'+appearance+'/';output+='<article><h3>'+esc(p.name)+' · '+esc(p.page)+' · '+esc(p.group)+'</h3>';const m=p.metrics;if(p.metrics){output+='<p>Content mismatch '+pct(m.content.mismatch_percent)+' · SSIM '+m.content.ssim?.toFixed(5)+' | Full mismatch '+pct(m.full.mismatch_percent)+' · SSIM '+m.full.ssim?.toFixed(5)+'</p>';for(const [name,r] of Object.entries(m.regions||{})){if(name!=='content')output+='<p>Region '+esc(name)+': '+(r.mismatch_percent===undefined?esc(r.error||'unavailable'):'mismatch '+pct(r.mismatch_percent)+' · SSIM '+r.ssim?.toFixed(5))+'</p>';}}if(p.baseline_delta){const d=p.baseline_delta;output+='<p>Baseline delta: '+(d.status==='compared'?'content mismatch '+d.content_mismatch_percent_points.toFixed(3)+' percentage points · SSIM '+(d.content_ssim?.toFixed(5)??'unavailable'):'unavailable (missing content metrics)')+'</p>';for(const [name,r] of Object.entries(d.regions||{}))output+='<p>Baseline delta '+esc(name)+': mismatch '+r.mismatch_percent_points.toFixed(3)+' percentage points · SSIM '+(r.ssim?.toFixed(5)??'unavailable')+'</p>';}if(p.error)output+='<p>'+esc(p.error)+'</p>';for(const app of Object.keys(p.captures).filter(a=>p.captures[a].size_pixels))output+='<a href="'+base+app+'.png">'+app+'</a>';if(p.metrics){output+='<a href="'+base+'heatmap.png">Heatmap</a><a href="'+base+'side-by-side.png">Side by side</a><a href="'+base+'overlay.png">50% overlay</a><div class="stack" style="width:'+m.full.size_pixels[0]+'px;aspect-ratio:'+m.full.size_pixels[0]+'/'+m.full.size_pixels[1]+'"><img style="width:'+(p.captures.gpui.size_pixels[0]/m.full.size_pixels[0]*100)+'%" src="'+base+'gpui.png"><img class="over" style="width:'+(p.captures.swiftui.size_pixels[0]/m.full.size_pixels[0]*100)+'%;opacity:'+document.getElementById('opacity').value/100+'" src="'+base+'swiftui.png"></div>';}else{for(const app of Object.keys(p.captures).filter(a=>p.captures[a].size_pixels))output+='<div><img src="'+base+app+'.png"></div>';}output+='</article>';}}document.getElementById('report').innerHTML=output;}document.getElementById('manifest').textContent=JSON.stringify(data.manifest,null,2);render(data.appearances[0]);</script></html>'''
    # Interactive overlays use the verified identical pixel canvas.
    (out / 'index.html').write_text(template.replace('__DATA__', data))


def parse_args():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--pages', nargs='+', default=PILOT, help='Inventory slugs, space or comma separated; "all" selects every page')
    parser.add_argument('--appearances', nargs='+', choices=['light', 'dark'], default=['light', 'dark'])
    parser.add_argument('--out', type=Path, default=ROOT / 'target/swiftui-compare')
    parser.add_argument('--overwrite', action='store_true', help='Allow reuse of a non-empty output directory')
    parser.add_argument('--baseline', type=Path, help='Previous results.json for content mismatch and SSIM deltas')
    parser.add_argument('--wait-idle', type=float, default=0, metavar='SECONDS', help='Wait for HID idle before taking focus; retry on input during settling (default off)')
    parser.add_argument('--settle', type=float, default=1.0)
    parser.add_argument('--window-timeout', type=float, default=30)
    parser.add_argument('--threshold', type=float, default=.1, help='YIQ tolerance 0..1 (default .1), includes antialias pixels')
    parser.add_argument('--regions', type=Path, default=ROOT / 'scripts/compare-swiftui/regions.json')
    parser.add_argument('--display', default='auto', help='auto: highest backing scale active display (default); main or CGDirectDisplayID')
    parser.add_argument('--bounds-only', action='store_true', help='Verify CGWindowList frame and placement without Screen Recording or pixel capture')
    parser.add_argument('--skip-build', action='store_true')
    parser.add_argument('--gallery-app', type=Path, help='Use this GPUI gallery .app instead of building target/debug (e.g. a baseline build); implies no gallery build')
    only = parser.add_mutually_exclusive_group()
    only.add_argument('--gallery-only', action='store_true')
    only.add_argument('--swiftui-only', action='store_true')
    args = parser.parse_args()
    if not math.isfinite(args.wait_idle) or args.wait_idle < 0:
        parser.error('wait-idle must be finite and >= 0')
    if not 0 <= args.threshold <= 1 or not math.isfinite(args.settle) or args.settle < 0 or not math.isfinite(args.window_timeout) or args.window_timeout <= 0:
        parser.error('threshold must be 0..1, settle >= 0 and window-timeout > 0 (finite)')
    return args


def main():
    # SIGTERM, like Ctrl-C, must unwind capture's PID cleanup.
    signal.signal(signal.SIGTERM, lambda *_: (_ for _ in ()).throw(KeyboardInterrupt()))
    args = parse_args()
    validate_output(args.out, args.overwrite)
    baseline = json.loads(args.baseline.read_text()) if args.baseline else None
    catalog = inventory()
    pages = list(dict.fromkeys(part for page in args.pages for part in page.split(',')))
    if pages == ['all']:
        pages = list(catalog)
    unknown = set(pages) - catalog.keys()
    if unknown:
        raise RuntimeError('Unknown inventory pages: ' + ', '.join(sorted(unknown)))
    config = json.loads(args.regions.read_text())
    regions = {page: page_regions(config, page) for page in pages}
    manifest = preflight(args.display, args.bounds_only)
    apps = ['swiftui'] if args.swiftui_only else ['gpui'] if args.gallery_only else ['gpui', 'swiftui']
    if not args.skip_build:
        for app in apps:
            if app == 'gpui' and args.gallery_app:
                continue
            script = ROOT / ('scripts/bundle-gallery.sh' if app == 'gpui' else 'scripts/build-swiftui-reference.sh')
            if not script.exists():
                raise RuntimeError(f'Build script missing: {script}; SwiftUI worker may still be building.')
            subprocess.run(['bash', str(script)], cwd=ROOT, check=True)
    executables = {}
    for app in apps:
        executables[app] = executable(args.gallery_app if app == 'gpui' and args.gallery_app else ROOT / ('target/debug/GPUI macOS Gallery.app' if app == 'gpui' else 'target/swiftui-reference/SwiftUI macOS Reference.app'), ROOT / 'target/debug/examples/gallery' if app == 'gpui' else None)
    args.out.mkdir(parents=True, exist_ok=True)
    results = {'manifest': manifest, 'metric_definition': {'mismatch': 'YIQ weighted squared distance > 35215 * threshold^2; all pixels including antialiasing; alpha composited onto white', 'ssim': 'scikit-image RGB channel mean, data_range=255, window up to 7 pixels', 'heatmap': 'red marks above-threshold mismatch; green shows normalized YIQ difference strength'}, 'pages': pages, 'appearances': args.appearances, 'threshold': args.threshold, 'settle_seconds': args.settle, 'regions_config': config, 'pairs': []}
    results['wait_idle_seconds'] = args.wait_idle
    if baseline is not None:
        results['baseline'] = str(args.baseline)
    failures = 0
    try:
        for page in pages:
            for appearance in args.appearances:
                folder = args.out / page / appearance
                folder.mkdir(parents=True, exist_ok=True)
                pair = {'page': page, 'appearance': appearance, **catalog[page], 'ranked': catalog[page]['category'] != 3, 'captures': {}}
                results['pairs'].append(pair)
                try:
                    for app in apps:
                        print(f'Capturing {app}: {page} / {appearance}', flush=True)
                        pair['captures'][app] = {}
                        capture(executables[app], page, appearance, folder / f'{app}.png', args.settle, args.window_timeout, pair['captures'][app], manifest['display'], args.bounds_only, args.wait_idle, extra_arguments=('--open',) if catalog.get(page, {}).get('open') else ())
                    if len(apps) == 2 and not args.bounds_only:
                        pair['metrics'] = compare(folder, pair['captures'], regions[page], args.threshold)
                        print(f"  content mismatch {pair['metrics']['content']['mismatch_percent']:.3f}%", flush=True)
                        for name, region in pair['metrics'].get('regions', {}).items():
                            if name != 'content' and 'mismatch_percent' in region:
                                print(f"  {name} mismatch {region['mismatch_percent']:.3f}%", flush=True)
                except (RuntimeError, subprocess.SubprocessError, OSError) as error:
                    pair['error'] = str(error)
                    failures += 1
                    print(f'  ERROR: {error}', file=sys.stderr, flush=True)
                if baseline is not None:
                    baseline_deltas(results, baseline)
                write_report(args.out, results)
    finally:
        if baseline is not None:
            baseline_deltas(results, baseline)
        write_report(args.out, results)
    if baseline is not None:
        print_baseline_deltas(results)
    print(f'Report: {args.out / "index.html"}; {len(results["pairs"])} pairs, {failures} failures')
    return 1 if failures else 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (RuntimeError, subprocess.SubprocessError, OSError, ValueError, KeyError) as error:
        print(f'compare-swiftui: {error}', file=sys.stderr)
        sys.exit(1)
