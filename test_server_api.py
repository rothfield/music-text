#!/usr/bin/env python3
"""
Comprehensive server-side test suite for music-text web API
Tests all endpoints with various notation systems and edge cases
"""

import requests
import json
import time
import sys
from typing import Dict, List, Any, Optional
from dataclasses import dataclass
from urllib.parse import quote

# Server configuration
BASE_URL = "http://127.0.0.1:3000"
PARSE_ENDPOINT = f"{BASE_URL}/api/parse"
LILYPOND_SVG_ENDPOINT = f"{BASE_URL}/api/lilypond-svg"

@dataclass
class TestCase:
    name: str
    input: str
    expected_success: bool
    expected_outputs: List[str]  # Which outputs should be present: pest, document, lily, vexflow
    description: str = ""

@dataclass
class TestResult:
    test_case: TestCase
    success: bool
    response_time_ms: float
    response_data: Dict[str, Any]
    errors: List[str]
    warnings: List[str]

class MusicTextAPITester:
    def __init__(self):
        self.results: List[TestResult] = []
        self.server_available = False
        
    def check_server_availability(self) -> bool:
        """Check if the server is running and responding"""
        try:
            response = requests.get(BASE_URL, timeout=5)
            self.server_available = response.status_code == 200
            print(f"✅ Server available at {BASE_URL}")
            return True
        except requests.exceptions.RequestException as e:
            print(f"❌ Server not available at {BASE_URL}: {e}")
            return False
    
    def test_parse_endpoint(self, test_case: TestCase) -> TestResult:
        """Test the /api/parse endpoint"""
        start_time = time.time()
        errors = []
        warnings = []
        response_data = {}
        
        try:
            # Make request with URL encoding
            url = f"{PARSE_ENDPOINT}?input={quote(test_case.input)}"
            response = requests.get(url, timeout=10)
            response_time_ms = (time.time() - start_time) * 1000
            
            # Parse JSON response
            response_data = response.json()
            
            # Check basic response structure
            if response.status_code != 200:
                errors.append(f"HTTP {response.status_code}: {response.reason}")
            
            # Validate success field
            actual_success = response_data.get('success', False)
            if actual_success != test_case.expected_success:
                errors.append(f"Expected success={test_case.expected_success}, got {actual_success}")
            
            # Check expected outputs are present
            for expected_output in test_case.expected_outputs:
                if expected_output not in response_data or response_data[expected_output] is None:
                    errors.append(f"Missing expected output: {expected_output}")
                elif expected_output in response_data and response_data[expected_output]:
                    # Output is present and non-empty
                    pass
            
            # Check for error field when success=False
            if not actual_success and not response_data.get('error'):
                warnings.append("Success=false but no error message provided")
            
            # Validate VexFlow structure if present
            if 'vexflow' in response_data and response_data['vexflow']:
                self._validate_vexflow_structure(response_data['vexflow'], warnings, errors)
            
            # Validate LilyPond output if present
            if 'minimal_lilypond' in response_data and response_data['minimal_lilypond']:
                self._validate_lilypond_output(response_data['minimal_lilypond'], warnings, errors)
                
        except requests.exceptions.Timeout:
            errors.append("Request timed out (>10s)")
            response_time_ms = 10000
        except requests.exceptions.RequestException as e:
            errors.append(f"Request failed: {e}")
            response_time_ms = (time.time() - start_time) * 1000
        except json.JSONDecodeError as e:
            errors.append(f"Invalid JSON response: {e}")
            response_time_ms = (time.time() - start_time) * 1000
        except Exception as e:
            errors.append(f"Unexpected error: {e}")
            response_time_ms = (time.time() - start_time) * 1000
        
        return TestResult(
            test_case=test_case,
            success=len(errors) == 0,
            response_time_ms=response_time_ms,
            response_data=response_data,
            errors=errors,
            warnings=warnings
        )
    
    def test_lilypond_svg_endpoint(self, input_notation: str) -> TestResult:
        """Test the /api/lilypond-svg endpoint"""
        test_case = TestCase("LilyPond SVG", input_notation, True, ["svg_content"])
        start_time = time.time()
        errors = []
        warnings = []
        response_data = {}
        
        try:
            payload = {"notation": input_notation}
            response = requests.post(LILYPOND_SVG_ENDPOINT, 
                                   json=payload, 
                                   headers={'Content-Type': 'application/json'},
                                   timeout=15)
            response_time_ms = (time.time() - start_time) * 1000
            
            response_data = response.json()
            
            if response.status_code != 200:
                errors.append(f"HTTP {response.status_code}: {response.reason}")
            
            actual_success = response_data.get('success', False)
            if not actual_success:
                error_msg = response_data.get('error', 'Unknown error')
                errors.append(f"SVG generation failed: {error_msg}")
            
            # Check SVG content
            svg_content = response_data.get('svg_content')
            if actual_success and svg_content:
                if not svg_content.startswith('<?xml') and not svg_content.startswith('<svg'):
                    warnings.append("SVG content doesn't start with expected XML/SVG tags")
                if len(svg_content) < 100:
                    warnings.append("SVG content seems unusually short")
            elif actual_success:
                errors.append("Success=true but no SVG content provided")
                
        except Exception as e:
            errors.append(f"Request error: {e}")
            response_time_ms = (time.time() - start_time) * 1000
        
        return TestResult(
            test_case=test_case,
            success=len(errors) == 0,
            response_time_ms=response_time_ms,
            response_data=response_data,
            errors=errors,
            warnings=warnings
        )
    
    def _validate_vexflow_structure(self, vexflow_data: Dict, warnings: List[str], errors: List[str]):
        """Validate VexFlow JSON structure"""
        if not isinstance(vexflow_data, dict):
            errors.append("VexFlow data is not a dictionary")
            return
        
        # Check required top-level fields
        required_fields = ['clef', 'key_signature', 'staves']
        for field in required_fields:
            if field not in vexflow_data:
                errors.append(f"Missing required VexFlow field: {field}")
        
        # Validate staves structure
        staves = vexflow_data.get('staves', [])
        if not isinstance(staves, list):
            errors.append("VexFlow staves should be a list")
        elif len(staves) == 0:
            warnings.append("VexFlow has no staves")
        else:
            for i, stave in enumerate(staves):
                if not isinstance(stave, dict):
                    errors.append(f"Stave {i} is not a dictionary")
                    continue
                
                if 'notes' not in stave:
                    errors.append(f"Stave {i} missing notes field")
                elif not isinstance(stave['notes'], list):
                    errors.append(f"Stave {i} notes should be a list")
                
                # Validate note structure
                notes = stave.get('notes', [])
                for j, note in enumerate(notes):
                    if not isinstance(note, dict):
                        errors.append(f"Stave {i} note {j} is not a dictionary")
                        continue
                    
                    if 'type' not in note:
                        errors.append(f"Stave {i} note {j} missing type field")
    
    def _validate_lilypond_output(self, lilypond_output: str, warnings: List[str], errors: List[str]):
        """Validate LilyPond output format"""
        if not lilypond_output.startswith('\\version'):
            warnings.append("LilyPond output doesn't start with \\version")
        
        if 'c4' not in lilypond_output and 'd4' not in lilypond_output and 'r4' not in lilypond_output:
            warnings.append("LilyPond output doesn't contain expected note patterns")
    
    def run_comprehensive_tests(self) -> Dict[str, Any]:
        """Run comprehensive test suite"""
        print("🧪 Starting comprehensive music-text API tests...")
        print("=" * 60)
        
        # Test cases covering various notation systems and edge cases
        test_cases = [
            # Basic notation systems
            TestCase("Simple number", "123", True, ["pest_output", "parsed_document", "minimal_lilypond", "vexflow"]),
            TestCase("Simple sargam", "SRG", True, ["pest_output", "parsed_document", "minimal_lilypond", "vexflow"]),
            TestCase("Simple western", "CDE", True, ["pest_output", "parsed_document", "minimal_lilypond", "vexflow"]),
            
            # With barlines
            TestCase("Number with barline", "|123", True, ["pest_output", "parsed_document", "minimal_lilypond", "vexflow"]),
            TestCase("Sargam with barline", "|SRG", True, ["pest_output", "parsed_document", "minimal_lilypond", "vexflow"]),
            
            # Complex rhythms (tuplets)
            TestCase("Simple tuplet", "|1-2", True, ["pest_output", "parsed_document", "minimal_lilypond", "vexflow"]),
            TestCase("Complex tuplet", "|1-2-3", True, ["pest_output", "parsed_document", "minimal_lilypond", "vexflow"]),
            
            # With dashes and extensions
            TestCase("Extended notes", "|1--2", True, ["pest_output", "parsed_document", "minimal_lilypond", "vexflow"]),
            TestCase("With rests", "|-1-", True, ["pest_output", "parsed_document", "minimal_lilypond", "vexflow"]),
            
            # Mixed notation systems
            TestCase("Mixed sargam/western", "SRmG", True, ["pest_output", "parsed_document", "minimal_lilypond", "vexflow"]),
            
            # Edge cases
            TestCase("Empty input", "", True, []),
            TestCase("Only spaces", "   ", True, []),
            TestCase("Only barline", "|", True, ["pest_output", "parsed_document", "minimal_lilypond", "vexflow"]),
            TestCase("Blank line before content", "\n123", True, ["pest_output", "parsed_document", "minimal_lilypond", "vexflow"]),
            
            # Error cases
            TestCase("Invalid characters", "xyz", False, ["pest_output"]),
            TestCase("Unmatched brackets", "[1 2 3", False, ["pest_output"]),
            
            # Stress tests
            TestCase("Long sequence", "1234567123456712345671234567", True, ["pest_output", "parsed_document", "minimal_lilypond", "vexflow"]),
            TestCase("Very long tuplet", "|1111111111111111111111111111111", True, ["pest_output", "parsed_document", "minimal_lilypond", "vexflow"]),
        ]
        
        # Run parse endpoint tests
        print("📡 Testing /api/parse endpoint...")
        parse_results = []
        for i, test_case in enumerate(test_cases, 1):
            print(f"  {i:2d}. Testing: {test_case.name} - '{test_case.input[:20]}{'...' if len(test_case.input) > 20 else ''}'")
            result = self.test_parse_endpoint(test_case)
            parse_results.append(result)
            
            # Print immediate feedback
            if result.success:
                print(f"      ✅ PASS ({result.response_time_ms:.1f}ms)")
            else:
                print(f"      ❌ FAIL ({result.response_time_ms:.1f}ms) - {'; '.join(result.errors)}")
                if result.warnings:
                    print(f"      ⚠️  WARN - {'; '.join(result.warnings)}")
        
        # Run LilyPond SVG endpoint tests
        print("\n🎼 Testing /api/lilypond-svg endpoint...")
        svg_test_cases = ["|123", "|SRG", "|1-2-3", "|CDE"]
        svg_results = []
        for i, notation in enumerate(svg_test_cases, 1):
            print(f"  {i}. Testing LilyPond SVG generation: '{notation}'")
            result = self.test_lilypond_svg_endpoint(notation)
            svg_results.append(result)
            
            if result.success:
                print(f"     ✅ PASS ({result.response_time_ms:.1f}ms)")
            else:
                print(f"     ❌ FAIL ({result.response_time_ms:.1f}ms) - {'; '.join(result.errors)}")
        
        # Compile results
        all_results = parse_results + svg_results
        total_tests = len(all_results)
        passed_tests = sum(1 for r in all_results if r.success)
        failed_tests = total_tests - passed_tests
        
        avg_response_time = sum(r.response_time_ms for r in all_results) / total_tests if total_tests > 0 else 0
        
        # Generate summary
        summary = {
            "total_tests": total_tests,
            "passed": passed_tests,
            "failed": failed_tests,
            "success_rate": (passed_tests / total_tests * 100) if total_tests > 0 else 0,
            "average_response_time_ms": avg_response_time,
            "parse_endpoint_tests": len(parse_results),
            "svg_endpoint_tests": len(svg_results),
            "detailed_results": all_results
        }
        
        self.results = all_results
        return summary
    
    def print_detailed_report(self, summary: Dict[str, Any]):
        """Print detailed test report"""
        print("\n" + "=" * 60)
        print("📊 DETAILED TEST REPORT")
        print("=" * 60)
        
        print(f"Total Tests: {summary['total_tests']}")
        print(f"Passed: {summary['passed']} ✅")
        print(f"Failed: {summary['failed']} ❌") 
        print(f"Success Rate: {summary['success_rate']:.1f}%")
        print(f"Average Response Time: {summary['average_response_time_ms']:.1f}ms")
        
        # Failed tests details
        failed_results = [r for r in self.results if not r.success]
        if failed_results:
            print(f"\n❌ FAILED TESTS ({len(failed_results)}):")
            for result in failed_results:
                print(f"  • {result.test_case.name}: {result.test_case.input}")
                print(f"    Errors: {'; '.join(result.errors)}")
                if result.warnings:
                    print(f"    Warnings: {'; '.join(result.warnings)}")
        
        # Performance analysis
        slow_tests = [r for r in self.results if r.response_time_ms > 1000]
        if slow_tests:
            print(f"\n🐌 SLOW TESTS (>1000ms):")
            for result in slow_tests:
                print(f"  • {result.test_case.name}: {result.response_time_ms:.1f}ms")
        
        # Response structure validation
        vexflow_tests = [r for r in self.results if r.success and 'vexflow' in r.response_data and r.response_data['vexflow']]
        print(f"\n🎵 VexFlow Structure Validation: {len(vexflow_tests)} tests with VexFlow data")
        
        lilypond_tests = [r for r in self.results if r.success and 'minimal_lilypond' in r.response_data and r.response_data['minimal_lilypond']]
        print(f"🎼 LilyPond Output Validation: {len(lilypond_tests)} tests with LilyPond data")
        
        print(f"\n⚡ Performance Summary:")
        print(f"  Fastest: {min(r.response_time_ms for r in self.results):.1f}ms")
        print(f"  Slowest: {max(r.response_time_ms for r in self.results):.1f}ms")
        print(f"  Median: {sorted([r.response_time_ms for r in self.results])[len(self.results)//2]:.1f}ms")

def main():
    """Main test execution function"""
    tester = MusicTextAPITester()
    
    # Check server availability
    if not tester.check_server_availability():
        print("❌ Cannot run tests - server is not available")
        print("Please ensure the server is running with: cargo run -- --web")
        sys.exit(1)
    
    # Run comprehensive tests
    summary = tester.run_comprehensive_tests()
    
    # Print detailed report
    tester.print_detailed_report(summary)
    
    # Exit with appropriate code
    if summary['success_rate'] == 100:
        print("\n🎉 All tests passed!")
        sys.exit(0)
    else:
        print(f"\n⚠️  {summary['failed']} tests failed")
        sys.exit(1)

if __name__ == "__main__":
    main()