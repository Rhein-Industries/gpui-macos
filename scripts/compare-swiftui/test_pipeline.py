"""Offline unit checks; run with the comparison script's Python dependencies.

python scripts/compare-swiftui/test_pipeline.py
No desktop capture or app launch: cleanup tests use a short-lived Python sleeper.
"""
import contextlib
import io
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import types
import unittest
from unittest.mock import Mock, patch

import numpy as np
from PIL import Image

SCRIPT = Path(__file__).resolve().parents[1] / 'compare-swiftui.py'
spec = importlib.util.spec_from_file_location('comparison', SCRIPT)
comparison = importlib.util.module_from_spec(spec)
# Avoid writing bytecode into the checkout when importing the standalone script.
exec(compile(SCRIPT.read_text(), str(SCRIPT), 'exec'), comparison.__dict__)


class PipelineTests(unittest.TestCase):
    def test_idle_polling_and_default_off(self):
        with patch.object(comparison.subprocess, 'check_output', side_effect=[
            '"HIDIdleTime" = 1000000000', '"HIDIdleTime" = 0',
            '"HIDIdleTime" = 120000000000',
        ]) as ioreg, patch.object(comparison.time, 'sleep') as sleep, contextlib.redirect_stdout(io.StringIO()) as output:
            comparison.wait_for_idle(0)
            ioreg.assert_not_called()
            comparison.wait_for_idle(120)
        self.assertEqual(ioreg.call_count, 3)
        self.assertEqual(ioreg.call_args.args[0], ['ioreg', '-c', 'IOHIDSystem'])
        self.assertEqual(sleep.call_count, 2)
        self.assertIn('idle >= 120s', output.getvalue())
        with patch.object(comparison, 'command', return_value='no idle property'):
            with self.assertRaisesRegex(RuntimeError, 'HIDIdleTime'):
                comparison.hid_idle_seconds()

    def test_settle_polls_and_rejects_input(self):
        clock = [0.0]
        def sleep(seconds):
            clock[0] += seconds
        with patch.object(comparison.time, 'monotonic', side_effect=lambda: clock[0]), patch.object(comparison.time, 'sleep', side_effect=sleep), patch.object(comparison, 'hid_idle_seconds', side_effect=[120, 121, 0]):
            with self.assertRaises(comparison.CaptureInterruptedByInput):
                comparison.settle_capture(1, 120)
        self.assertAlmostEqual(clock[0], .2)
        with patch.object(comparison.time, 'monotonic', side_effect=lambda: clock[0]), patch.object(comparison.time, 'sleep', side_effect=sleep), patch.object(comparison, 'hid_idle_seconds', return_value=5):
            comparison.settle_capture(.2, 120)
        with patch.object(comparison, 'hid_idle_seconds') as idle, patch.object(comparison.time, 'sleep') as sleep:
            comparison.settle_capture(1, 0)
            idle.assert_not_called()
            sleep.assert_called_once_with(1)

    def test_capture_retries_after_input_and_restores_focus(self):
        self.check_mock_capture('retry')

    def test_keyboard_interrupt_reaps_pid_and_restores_focus(self):
        for phase in ['window', 'settle', 'screenshot']:
            with self.subTest(phase=phase):
                self.check_mock_capture(phase)

    def check_mock_capture(self, phase):
        display = {'id': 2, 'name': 'Retina', 'scale': 2,
                   'bounds_points': {'X': 0, 'Y': 0, 'Width': 1512, 'Height': 982}}
        window = {'id': 42, 'bounds': {'X': 100, 'Y': 100, 'Width': 1060, 'Height': 760}}
        cursor_restores = []
        quartz = types.SimpleNamespace(kCGWindowNumber='id', kCGWindowBounds='bounds', kCGWindowName='title',
                                       CGDisplayHideCursor=lambda _: 0, CGDisplayShowCursor=lambda _: cursor_restores.append(True))
        processes = []
        def launch(*args, **kwargs):
            process = Mock(pid=100 + len(processes), returncode=None)
            process.poll.side_effect = lambda: process.returncode
            process.wait.side_effect = lambda **kwargs: setattr(process, 'returncode', -15)
            processes.append(process)
            return process
        clock = [0.0]
        def sleep(seconds):
            clock[0] += seconds
        idle_values = iter([120, 120, 0, 1, 120, 120, 121, 122])
        previous = Mock()
        record = {}
        with tempfile.TemporaryDirectory() as tmp, contextlib.ExitStack() as stack:
            destination = Path(tmp) / 'capture.png'
            destination.write_bytes(b'discard me')
            stack.enter_context(patch.dict(sys.modules, {'Quartz': quartz}))
            stack.enter_context(patch.object(comparison, 'frontmost_application', return_value=previous))
            restore = stack.enter_context(patch.object(comparison, 'restore_focus'))
            stack.enter_context(patch.object(comparison, 'is_frontmost', return_value=True))
            stack.enter_context(patch.object(comparison, 'main_window', side_effect=KeyboardInterrupt() if phase == 'window' else lambda _: window))
            stack.enter_context(patch.object(comparison.subprocess, 'Popen', side_effect=launch))
            screenshot = stack.enter_context(patch.object(comparison.subprocess, 'run', side_effect=KeyboardInterrupt()))
            stack.enter_context(patch.object(comparison.time, 'sleep', side_effect=sleep))
            stack.enter_context(patch.object(comparison.time, 'monotonic', side_effect=lambda: clock[0]))
            if phase == 'retry':
                stack.enter_context(patch.object(comparison.subprocess, 'check_output', side_effect=lambda *a, **k: f'"HIDIdleTime" = {next(idle_values, 123) * 1_000_000_000}'))
                comparison.capture(Path('/fixture'), 'checkbox', 'light', destination, .2, 1, record, display, bounds_only=True, wait_idle=120)
                self.assertFalse(destination.exists())
                self.assertEqual(len(processes), 2)
                screenshot.assert_not_called()
            else:
                if phase == 'settle':
                    stack.enter_context(patch.object(comparison, 'settle_capture', side_effect=KeyboardInterrupt()))
                with self.assertRaises(KeyboardInterrupt):
                    comparison.capture(Path('/fixture'), 'checkbox', 'light', destination, 0, 1, record, display)
            self.assertEqual(restore.call_count, len(processes))
            restore.assert_called_with(previous, record)
        for process in processes:
            process.terminate.assert_called_once()
            process.wait.assert_called_once_with(timeout=5)
        self.assertTrue(record['process_reaped'])
        self.assertEqual(cursor_restores, [True] if phase == 'screenshot' else [])

    def test_output_preservation_and_overwrite(self):
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / 'output'
            comparison.validate_output(out)
            out.mkdir()
            comparison.validate_output(out)
            sentinel = out / 'results.json'
            sentinel.write_text('keep')
            with self.assertRaisesRegex(RuntimeError, '--overwrite'):
                comparison.validate_output(out)
            comparison.validate_output(out, overwrite=True)
            self.assertEqual(sentinel.read_text(), 'keep')
            with self.assertRaises(RuntimeError):
                comparison.validate_output(sentinel, overwrite=True)
            with patch.object(sys, 'argv', ['compare', '--out', str(out)]), patch.object(comparison, 'preflight') as preflight:
                with self.assertRaises(RuntimeError):
                    comparison.main()
                preflight.assert_not_called()

    def test_new_arguments(self):
        with patch.object(sys, 'argv', ['compare']):
            args = comparison.parse_args()
            self.assertEqual(args.wait_idle, 0)
            self.assertFalse(args.overwrite)
            self.assertIsNone(args.baseline)
        with patch.object(sys, 'argv', ['compare', '--wait-idle', '120', '--overwrite', '--baseline', '/tmp/results.json']):
            args = comparison.parse_args()
            self.assertEqual(args.wait_idle, 120)
            self.assertTrue(args.overwrite)
            self.assertEqual(args.baseline, Path('/tmp/results.json'))
        for value in ['-1', 'nan', 'inf']:
            with patch.object(sys, 'argv', ['compare', '--wait-idle', value]), contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
                comparison.parse_args()

    def test_baseline_deltas_json_html_and_stdout(self):
        def pair(page, appearance, mismatch, ssim):
            return {'page': page, 'appearance': appearance, 'name': page, 'category': 1, 'group': 'Forms', 'captures': {},
                    'metrics': {'content': {'mismatch_percent': mismatch, 'ssim': ssim}}}
        results = {'manifest': {}, 'appearances': ['light', 'dark'], 'pairs': [
            pair('checkbox', 'light', 3, .92), pair('checkbox', 'dark', 5, .85),
            pair('switch', 'light', 2, None), pair('button', 'light', 1, .9),
        ]}
        baseline = {'pairs': [pair('checkbox', 'dark', 4, .86), pair('checkbox', 'light', 4, .9), pair('switch', 'light', 2, .9)]}
        comparison.baseline_deltas(results, baseline)
        deltas = [p['baseline_delta'] for p in results['pairs']]
        self.assertEqual(deltas[0]['content_mismatch_percent_points'], -1)
        self.assertAlmostEqual(deltas[0]['content_ssim'], .02)
        self.assertEqual(deltas[1]['content_mismatch_percent_points'], 1)
        self.assertAlmostEqual(deltas[1]['content_ssim'], -.01)
        self.assertIsNone(deltas[2]['content_ssim'])
        self.assertEqual(deltas[3]['status'], 'unavailable')
        with contextlib.redirect_stdout(io.StringIO()) as output:
            comparison.print_baseline_deltas(results)
        self.assertIn('checkbox / light: content mismatch -1.000 percentage points; SSIM +0.02000', output.getvalue())
        self.assertIn('button / light: unavailable', output.getvalue())
        with tempfile.TemporaryDirectory() as tmp:
            comparison.write_report(Path(tmp), results)
            self.assertEqual(json.loads((Path(tmp) / 'results.json').read_text()), results)
            report = (Path(tmp) / 'index.html').read_text()
            self.assertIn('Baseline delta:', report)
            self.assertIn('content_mismatch_percent_points', report)
            self.assertIn('content_ssim', report)

    def test_main_baseline_and_interrupt_reports(self):
        for interrupted in [False, True]:
            with self.subTest(interrupted=interrupted), tempfile.TemporaryDirectory() as tmp:
                out = Path(tmp) / 'run'
                baseline = Path(tmp) / 'baseline.json'
                baseline.write_text(json.dumps({'pairs': [{'page': 'checkbox', 'appearance': 'light',
                    'metrics': {'content': {'mismatch_percent': 4, 'ssim': .9}}}]}))
                argv = ['compare', '--skip-build', '--pages', 'checkbox', '--appearances', 'light',
                        '--out', str(out), '--baseline', str(baseline), '--wait-idle', '120']
                with patch.object(sys, 'argv', argv), patch.object(comparison, 'preflight', return_value={'display': {'id': 2}}), patch.object(comparison, 'executable', return_value=Path('/fixture')), patch.object(comparison, 'capture', side_effect=KeyboardInterrupt() if interrupted else None) as capture, patch.object(comparison, 'compare', return_value={'content': {'mismatch_percent': 3, 'ssim': .92}}), contextlib.redirect_stdout(io.StringIO()) as output:
                    if interrupted:
                        with self.assertRaises(KeyboardInterrupt):
                            comparison.main()
                    else:
                        self.assertEqual(comparison.main(), 0)
                        self.assertIn('Baseline delta checkbox / light', output.getvalue())
                    self.assertEqual(capture.call_args.args[-1], 120)
                results = json.loads((out / 'results.json').read_text())
                self.assertEqual(results['baseline'], str(baseline))
                self.assertEqual(results['wait_idle_seconds'], 120)
                delta = results['pairs'][0]['baseline_delta']
                self.assertEqual(delta['status'], 'unavailable' if interrupted else 'compared')
                if not interrupted:
                    self.assertEqual(delta['content_mismatch_percent_points'], -1)
                self.assertIn('Baseline delta:', (out / 'index.html').read_text())

    def test_inventory(self):
        catalog = comparison.inventory()
        self.assertEqual(len(catalog), 100)
        self.assertEqual([sum(p['category'] == c for p in catalog.values()) for c in [1, 2, 3]], [51, 42, 7])
        self.assertTrue(all(catalog[p]['category'] == 1 for p in comparison.PILOT))

    def test_metrics_and_artifacts(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp)
            a = np.full((80, 100, 3), 255, dtype=np.uint8)
            Image.fromarray(a).save(folder / 'gpui.png')
            Image.fromarray(a).save(folder / 'swiftui.png')
            captures = {app: {'size_pixels': [100, 80], 'capture_scale': [2, 2]} for app in ['gpui', 'swiftui']}
            region = {'x': 10, 'y': 10}
            result = comparison.compare(folder, captures, region, .1)
            self.assertEqual(result['full']['mismatch_percent'], 0)
            self.assertEqual(result['full']['ssim'], 1)
            self.assertEqual(result['content']['box_pixels'], [20, 20, 100, 80])
            inset = comparison.compare(folder, captures, {'x': 10, 'y': 10, 'right': 5, 'bottom': 3}, .1)
            self.assertEqual(inset['content']['box_pixels'], [20, 20, 90, 74])
            b = a.copy()
            b[:20] = 0  # Difference entirely outside the content crop.
            Image.fromarray(b).save(folder / 'swiftui.png')
            result = comparison.compare(folder, captures, region, .1)
            self.assertEqual(result['full']['mismatch_percent'], 25)
            self.assertEqual(result['content']['mismatch_percent'], 0)
            self.assertLess(result['full']['ssim'], 1)
            Image.fromarray(b[:70, :90]).save(folder / 'swiftui.png')
            captures['swiftui']['size_pixels'] = [90, 70]
            with self.assertRaisesRegex(RuntimeError, 'mismatched size_pixels'):
                comparison.compare(folder, captures, region, .1)
            for name, size in [('heatmap', (100, 80)), ('overlay', (100, 80)), ('side-by-side', (200, 80))]:
                with Image.open(folder / f'{name}.png') as image:
                    self.assertEqual(image.size, size)
            data = {'manifest': {}, 'appearances': ['light'], 'pairs': [{'page': 'checkbox', 'name': '<test>', 'group': 'Forms', 'category': 1, 'appearance': 'light', 'captures': captures, 'metrics': result}]}
            comparison.write_report(folder, data)
            self.assertEqual(json.loads((folder / 'results.json').read_text()), data)
            self.assertIn('opacity', (folder / 'index.html').read_text())
            self.assertNotIn('<test>', (folder / 'index.html').read_text())

    def test_page_regions_merge_named_regions_and_validate(self):
        config = json.loads((SCRIPT.parent / 'compare-swiftui' / 'regions.json').read_text())
        regions = comparison.page_regions(config, 'checkbox')
        self.assertEqual(regions['content'], {'x': 220, 'y': 52, 'right': 0, 'bottom': 0})
        self.assertEqual(regions['sidebar'], {'x': 0, 'y': 52, 'width': 220, 'bottom': 0})
        config = {'default': {'x': 220, 'y': 52}, 'regions': {'sidebar': {'x': 0, 'y': 52, 'width': 220}},
                  'pages': {'table': {'y': 60, 'regions': {'sidebar': {'y': 70}, 'header': {'x': 220, 'y': 52, 'height': 27}}}}}
        regions = comparison.page_regions(config, 'table')
        self.assertEqual(regions['content'], {'x': 220, 'y': 60})
        self.assertEqual(regions['sidebar'], {'x': 0, 'y': 70, 'width': 220})
        self.assertEqual(regions['header'], {'x': 220, 'y': 52, 'height': 27})
        self.assertEqual(set(comparison.page_regions(config, 'checkbox')), {'content', 'sidebar'})
        for bad in [{'default': {'x': -1, 'y': 0}}, {'default': {'x': 0}},
                    {'default': {'x': 0, 'y': 0}, 'regions': {'sidebar': {'x': 0, 'y': 0, 'depth': 3}}},
                    {'default': {'x': 0, 'y': 0}, 'regions': {'sidebar': {'x': float('nan'), 'y': 0}}}]:
            with self.assertRaises(RuntimeError):
                comparison.page_regions(bad, 'checkbox')

    def test_named_regions_from_one_capture(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp)
            a = np.full((80, 100, 3), 255, dtype=np.uint8)
            b = a.copy()
            b[20:, :20] = 0  # Difference only inside the sidebar (x 0-10 pt, below y 10 pt).
            Image.fromarray(a).save(folder / 'gpui.png')
            Image.fromarray(b).save(folder / 'swiftui.png')
            captures = {app: {'size_pixels': [100, 80], 'capture_scale': [2, 2]} for app in ['gpui', 'swiftui']}
            regions = {'content': {'x': 10, 'y': 10}, 'sidebar': {'x': 0, 'y': 10, 'width': 10, 'bottom': 0},
                       'empty': {'x': 50, 'y': 10, 'width': 0}}
            result = comparison.compare(folder, captures, regions, .1)
            # The content metric keeps its pre-region shape (baselines read it).
            self.assertEqual(result['content']['box_pixels'], [20, 20, 100, 80])
            self.assertNotIn('region_points', result['content'])
            self.assertEqual(result['content']['mismatch_percent'], 0)
            self.assertEqual(result['region_points'], regions['content'])
            sidebar = result['regions']['sidebar']
            self.assertEqual(sidebar['box_pixels'], [0, 20, 20, 80])
            self.assertEqual(sidebar['mismatch_percent'], 100)
            self.assertEqual(sidebar['region_points'], regions['sidebar'])
            self.assertEqual(result['regions']['content']['box_pixels'], [20, 20, 100, 80])
            self.assertIn('error', result['regions']['empty'])
            self.assertEqual(comparison.compare(folder, captures, {'x': 10, 'y': 10}, .1)['content'], result['content'])
            with self.assertRaisesRegex(RuntimeError, 'Content region is empty'):
                comparison.compare(folder, captures, {'content': {'x': 10, 'y': 10, 'width': 0}}, .1)
            data = {'manifest': {}, 'appearances': ['light'], 'pairs': [{'page': 'checkbox', 'name': 'Checkbox', 'group': 'Forms', 'category': 1, 'appearance': 'light', 'captures': captures, 'metrics': result}]}
            comparison.write_report(folder, data)
            self.assertIn("Region '+esc(name)", (folder / 'index.html').read_text())
            self.assertIn("Baseline delta '+esc(name)", (folder / 'index.html').read_text())

    def test_region_baseline_deltas(self):
        def pair(content, sidebar=None):
            metrics = {'content': {'mismatch_percent': content, 'ssim': .9}}
            if sidebar is not None:
                metrics['regions'] = {'content': metrics['content'], 'sidebar': {'mismatch_percent': sidebar, 'ssim': .8}}
            return {'page': 'checkbox', 'appearance': 'light', 'metrics': metrics}
        for prior, expected in [(pair(4, 6), {'sidebar': {'mismatch_percent_points': -4, 'ssim': 0}}), (pair(4), None)]:
            results = {'pairs': [pair(3, 2)]}
            comparison.baseline_deltas(results, {'pairs': [prior]})
            delta = results['pairs'][0]['baseline_delta']
            self.assertEqual(delta['content_mismatch_percent_points'], -1)
            self.assertEqual(delta.get('regions'), expected)
            with contextlib.redirect_stdout(io.StringIO()) as output:
                comparison.print_baseline_deltas(results)
            self.assertEqual('sidebar mismatch -4.000' in output.getvalue(), expected is not None)

    def test_main_passes_every_named_region(self):
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / 'run'
            argv = ['compare', '--skip-build', '--pages', 'checkbox', '--appearances', 'light', '--out', str(out)]
            metrics = {'content': {'mismatch_percent': 3, 'ssim': .9}, 'regions': {'sidebar': {'mismatch_percent': 1.5, 'ssim': .95}}}
            with patch.object(sys, 'argv', argv), patch.object(comparison, 'preflight', return_value={'display': {'id': 2}}), patch.object(comparison, 'executable', return_value=Path('/fixture')), patch.object(comparison, 'capture'), patch.object(comparison, 'compare', return_value=metrics) as compare, contextlib.redirect_stdout(io.StringIO()) as output:
                self.assertEqual(comparison.main(), 0)
            self.assertEqual(set(compare.call_args.args[2]), {'content', 'sidebar'})
            self.assertIn('sidebar mismatch 1.500%', output.getvalue())

    def test_display_selection(self):
        main = {'id': 1, 'name': 'DELL', 'scale': 1}
        retina = {'id': 2, 'name': 'Color LCD', 'scale': 2}
        displays = [main, retina]
        self.assertEqual(comparison.select_display(displays, 'auto', 1), retina)
        self.assertEqual(comparison.select_display(displays, 'main', 1), main)
        self.assertEqual(comparison.select_display(displays, '2', 1), retina)
        for requested in ['999', 'bogus']:
            with self.assertRaises(RuntimeError):
                comparison.select_display(displays, requested, 1)

    def test_active_display_metadata_uses_backing_scale(self):
        screen = types.SimpleNamespace(deviceDescription=lambda: {'NSScreenNumber': 2},
                                       localizedName=lambda: 'Color LCD', backingScaleFactor=lambda: 2)
        rect = types.SimpleNamespace(origin=types.SimpleNamespace(x=3840, y=431),
                                     size=types.SimpleNamespace(width=1512, height=982))
        quartz = types.SimpleNamespace(CGGetActiveDisplayList=lambda *_: (0, [2], 1),
                                      CGDisplayBounds=lambda _: rect)
        appkit = types.SimpleNamespace(NSScreen=types.SimpleNamespace(screens=lambda: [screen]))
        with patch.dict(sys.modules, {'Quartz': quartz, 'AppKit': appkit}):
            display, = comparison.active_displays()
        self.assertEqual((display['id'], display['name'], display['scale']), (2, 'Color LCD', 2))
        self.assertEqual(display['bounds_points']['X'], 3840)
        quartz.CGGetActiveDisplayList = lambda *_: (0, [], 0)
        with patch.dict(sys.modules, {'Quartz': quartz}), self.assertRaisesRegex(RuntimeError, 'No active displays visible'):
            comparison.active_displays()

    def test_window_and_backing_scale_checks(self):
        display = {'id': 2, 'name': 'Color LCD', 'scale': 2,
                   'bounds_points': {'X': -1512, 'Y': -982, 'Width': 1512, 'Height': 982}}
        bounds = {'X': -1300, 'Y': -900, 'Width': 1060, 'Height': 760}
        comparison.validate_window(bounds, display)
        for bad, error in [({**bounds, 'Width': 1061}, '1061x760'),
                           ({**bounds, 'X': 0}, 'not wholly on display'),
                           ({**bounds, 'X': -500}, 'not wholly on display')]:
            with self.assertRaisesRegex(RuntimeError, error):
                comparison.validate_window(bad, display)
        comparison.validate_capture({'size_pixels': [2120, 1520], 'capture_scale': [2, 2]}, display)
        for record in [{'size_pixels': [1060, 760], 'capture_scale': [1, 1]},
                       {'size_pixels': [2120, 1520], 'capture_scale': [2, 1]}]:
            with self.assertRaisesRegex(RuntimeError, 'expected .* at 2x'):
                comparison.validate_capture(record, display)

    def test_pair_rejects_scale_and_point_size_mismatch(self):
        a = {'size_pixels': [2120, 1520], 'capture_scale': [2, 2],
             'window_bounds_points': {'Width': 1060, 'Height': 760}}
        comparison.validate_pair({'gpui': a, 'swiftui': dict(a)})
        for b, error in [({**a, 'capture_scale': [1, 1]}, 'capture_scale'),
                         ({**a, 'window_bounds_points': {'Width': 1061, 'Height': 760}}, 'point sizes')]:
            with self.assertRaisesRegex(RuntimeError, error):
                comparison.validate_pair({'gpui': a, 'swiftui': b})

    def test_threshold(self):
        a = np.full((10, 10, 3), 255, dtype=np.uint8)
        b = a - 1
        delta = comparison.color_delta(a, b)
        self.assertEqual(comparison.metrics(a, b, delta, .1)['mismatch_percent'], 0)
        self.assertEqual(comparison.metrics(a, b, delta, 0)['mismatch_percent'], 100)

    def test_capture_failure_reaps_exact_process_and_restores_cursor(self):
        fake_q = types.SimpleNamespace(CGMainDisplayID=lambda: 1, CGDisplayHideCursor=lambda _: 0, CGDisplayShowCursor=lambda _: restored.append(True), kCGWindowNumber='id', kCGWindowBounds='bounds', kCGWindowName='title')
        window = {'id': 42, 'bounds': {'X': 0, 'Y': 0, 'Width': 1060, 'Height': 760}, 'title': 'Fixture'}
        display = {'id': 2, 'name': 'Retina', 'scale': 2, 'bounds_points': {'X': 0, 'Y': 0, 'Width': 1512, 'Height': 982}}
        restored, launched = [], []
        real_popen = subprocess.Popen
        def launch(*args, **kwargs):
            process = real_popen([sys.executable, '-c', 'import time; time.sleep(30)'], **kwargs)
            launched.append(process)
            return process
        record = {}
        with tempfile.TemporaryDirectory() as tmp, patch.dict(sys.modules, {'Quartz': fake_q}), patch.object(comparison, 'frontmost_application', return_value=None), patch.object(comparison, 'is_frontmost', return_value=True), patch.object(comparison, 'main_window', return_value=window), patch.object(comparison.subprocess, 'Popen', side_effect=launch), patch.object(comparison.subprocess, 'run', side_effect=subprocess.CalledProcessError(1, 'screencapture')):
            with self.assertRaises(subprocess.CalledProcessError):
                comparison.capture(Path('/fixture'), 'checkbox', 'light', Path(tmp) / 'gpui.png', 0, 1, record, display)
        self.assertEqual(record['pid'], launched[0].pid)
        self.assertTrue(record['process_reaped'])
        self.assertIsNotNone(launched[0].poll())
        self.assertEqual(restored, [True])

    def test_bounds_only_skips_screenshot_and_reaps_pid(self):
        display = {'id': 2, 'name': 'Color LCD', 'scale': 2,
                   'bounds_points': {'X': 0, 'Y': 0, 'Width': 1512, 'Height': 982}}
        quartz = types.SimpleNamespace(kCGWindowNumber='id', kCGWindowBounds='bounds', kCGWindowName='title')
        window = {'id': 42, 'bounds': {'X': 100, 'Y': 100, 'Width': 1060, 'Height': 760}}
        real_popen = subprocess.Popen
        launched = []
        def launch(*args, **kwargs):
            process = real_popen([sys.executable, '-c', 'import time; time.sleep(30)'], **kwargs)
            launched.append(process)
            return process
        record = {}
        with tempfile.TemporaryDirectory() as tmp, patch.dict(sys.modules, {'Quartz': quartz}), patch.object(comparison, 'frontmost_application', return_value=None), patch.object(comparison, 'is_frontmost', return_value=True), patch.object(comparison, 'main_window', return_value=window), patch.object(comparison.subprocess, 'Popen', side_effect=launch), patch.object(comparison.subprocess, 'run') as screenshot:
            comparison.capture(Path('/fixture'), 'checkbox', 'light', Path(tmp) / 'gpui.png', 0, 1, record, display, bounds_only=True)
            screenshot.assert_not_called()
        self.assertEqual(record['display_backing_scale'], 2)
        self.assertNotIn('capture_scale', record)
        self.assertTrue(record['process_reaped'])
        self.assertIsNotNone(launched[0].poll())

    def test_invalid_window_fails_before_capture_and_reaps_pid(self):
        display = {'id': 2, 'name': 'Color LCD', 'scale': 2,
                   'bounds_points': {'X': 3840, 'Y': 431, 'Width': 1512, 'Height': 982}}
        quartz = types.SimpleNamespace(kCGWindowNumber='id', kCGWindowBounds='bounds', kCGWindowName='title')
        bad_window = {'id': 42, 'bounds': {'X': 4000, 'Y': 500, 'Width': 1061, 'Height': 760}}
        real_popen = subprocess.Popen
        launched, arguments = [], []
        def launch(args, **kwargs):
            arguments.extend(args)
            process = real_popen([sys.executable, '-c', 'import time; time.sleep(30)'], **kwargs)
            launched.append(process)
            return process
        record = {}
        with tempfile.TemporaryDirectory() as tmp, patch.dict(sys.modules, {'Quartz': quartz}), patch.object(comparison, 'frontmost_application', return_value=None), patch.object(comparison, 'is_frontmost', return_value=True), patch.object(comparison, 'main_window', return_value=bad_window), patch.object(comparison.subprocess, 'Popen', side_effect=launch), patch.object(comparison.subprocess, 'run') as screenshot:
            with self.assertRaisesRegex(RuntimeError, '1061x760'):
                comparison.capture(Path('/fixture'), 'checkbox', 'light', Path(tmp) / 'gpui.png', 0, 1, record, display)
            screenshot.assert_not_called()
        self.assertEqual(arguments, ['/fixture', 'checkbox', '--light', '--display', '2'])
        self.assertTrue(record['process_reaped'])
        self.assertIsNotNone(launched[0].poll())


if __name__ == '__main__':
    unittest.main()
