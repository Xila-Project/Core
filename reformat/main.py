#!/usr/bin/env python3
"""
Script to interact with rust-analyzer to identify and fix non-snake_case warnings.
"""

import logging
import re
import subprocess
from pathlib import Path
from typing import List
from dataclasses import dataclass

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class SnakeCaseWarning:
    """Represents a non-snake_case warning from rust-analyzer."""
    file_path: str
    line: int
    col: int
    end_line: int
    end_col: int
    message: str
    suggestion: str
    warning_type: str  # e.g., "non_snake_case", "non_upper_case_globals"

def parse_rust_analyzer_output(output: str) -> List[SnakeCaseWarning]:
    """Parse rust-analyzer diagnostics output to extract non-snake_case warnings."""
    warnings = []
    
    # Pattern to match warning lines
    warning_pattern = re.compile(
        r'Warning RustcLint\("([^"]+)"\) from LineCol \{ line: (\d+), col: (\d+) \} to LineCol \{ line: (\d+), col: (\d+) \}: (.+)'
    )
    
    # Pattern to match processing crate lines to get the current file
    crate_pattern = re.compile(r'processing crate: [^,]+, module: (.+)')
    
    current_file = None
    
    for line in output.splitlines():
        line = line.strip()
        
        # Check for crate processing line
        crate_match = crate_pattern.match(line)
        if crate_match:
            current_file = crate_match.group(1)
            continue
        
        # Check for warning line
        warning_match = warning_pattern.match(line)
        if warning_match and current_file:
            warning_type = warning_match.group(1)
            start_line = int(warning_match.group(2))
            start_col = int(warning_match.group(3))
            end_line = int(warning_match.group(4))
            end_col = int(warning_match.group(5))
            message = warning_match.group(6)
            
            # Extract suggestion from message
            suggestion = ""
            if "e.g." in message:
                suggestion = message.split("e.g. `")[1].rstrip("`")
            
            # Only process snake_case related warnings
            if warning_type in ["non_snake_case", "non_upper_case_globals"]:
                warning = SnakeCaseWarning(
                    file_path=current_file,
                    line=start_line,
                    col=start_col,
                    end_line=end_line,
                    end_col=end_col,
                    message=message,
                    suggestion=suggestion,
                    warning_type=warning_type
                )
                warnings.append(warning)
    
    return warnings



def should_skip_warning(warning: SnakeCaseWarning) -> bool:
    """
    Determine if a warning should be skipped for automatic fixing.
    Some warnings are too risky to fix automatically.
    """
    # Skip if no suggestion
    if not warning.suggestion:
        return True
    
    # Skip warnings for keywords that might conflict with Rust keywords
    rust_keywords = {
        'as', 'break', 'const', 'continue', 'crate', 'else', 'enum', 'extern',
        'false', 'fn', 'for', 'if', 'impl', 'in', 'let', 'loop', 'match',
        'mod', 'move', 'mut', 'pub', 'ref', 'return', 'self', 'Self',
        'static', 'struct', 'super', 'trait', 'true', 'type', 'unsafe',
        'use', 'where', 'while', 'async', 'await', 'dyn'
    }
    
    if warning.suggestion in rust_keywords:
        return True
    
    # Skip very short lines or positions that seem problematic
    try:
        with open(warning.file_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        
        if warning.line - 1 >= len(lines):
            return True
        
        line = lines[warning.line - 1]
        
        # Skip if line is too short or position is clearly wrong
        if len(line.strip()) < 3 or warning.col > len(line):
            return False
            return True
        
        # Skip if this looks like it's in a comment or string
        line_before_pos = line[:warning.col - 1]
        if '//' in line_before_pos or '/*' in line_before_pos:
            return True
        
        # Count quotes to see if we're inside a string
        single_quotes = line_before_pos.count("'") - line_before_pos.count("\\'")
        double_quotes = line_before_pos.count('"') - line_before_pos.count('\\"')
        
        if single_quotes % 2 == 1 or double_quotes % 2 == 1:
            return True
        
    except Exception:
        # If we can't read the file, skip this warning
        return True
    
    return False
    
class RustAnalyzerLSPClient:
    """A reusable LSP client for rust-analyzer."""
    
    def __init__(self, workspace_root: str):
        self.workspace_root = workspace_root
        self.process = None
        self.request_id = 0
        self.opened_documents: set[str] = set()
        
    def start(self):
        """Start the rust-analyzer LSP server."""
        import os
        
        self.process = subprocess.Popen(
            ["rust-analyzer"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            cwd=self.workspace_root
        )
        
        # Initialize the LSP session
        self.request_id += 1
        initialize_request = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": "initialize",
            "params": {
                "processId": os.getpid(),
                "rootUri": f"file://{Path(self.workspace_root).resolve()}",
                "capabilities": {
                    "textDocument": {
                        "rename": {
                            "prepareSupport": True
                        }
                    }
                }
            }
        }
        
        self.send_request(initialize_request)
        init_response = self.read_response()
        
        if init_response.get("id") != self.request_id:
            logger.error("Failed to initialize LSP session")
            return False
        
        # Send initialized notification
        self.send_request({"jsonrpc": "2.0", "method": "initialized", "params": {}})
        return True
        
    def send_request(self, request):
        """Send an LSP request to rust-analyzer."""
        import json
        content = json.dumps(request)
        message = f"Content-Length: {len(content)}\r\n\r\n{content}"
        self.process.stdin.write(message)
        self.process.stdin.flush()
        
    def read_response(self):
        """Read an LSP response from rust-analyzer, skipping notifications."""
        import json
        while True:
            # Read the Content-Length header
            while True:
                line = self.process.stdout.readline()
                if line.startswith("Content-Length:"):
                    length = int(line.split(":")[1].strip())
                    break
            
            # Read the empty line
            self.process.stdout.readline()
            
            # Read the JSON content
            content = self.process.stdout.read(length)
            message = json.loads(content)
            
            # If this is a notification (no id), skip it and read the next message
            if "id" not in message:
                logger.debug(f"Skipping notification: {message.get('method', 'unknown')}")
                continue
            
            # This is a response, return it
            return message
            
    def update_document(self, file_path: str):
        """Update the document content in the LSP session after file changes."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                file_content = f.read()
            
            # Send didChange notification to update the document
            did_change_request = {
                "jsonrpc": "2.0",
                "method": "textDocument/didChange",
                "params": {
                    "textDocument": {
                        "uri": f"file://{Path(file_path).resolve()}",
                        "version": 2  # Increment version number
                    },
                    "contentChanges": [
                        {
                            "text": file_content  # Full text replacement
                        }
                    ]
                }
            }
            
            self.send_request(did_change_request)
            
        except Exception as e:
            logger.error(f"Error updating document {file_path}: {e}")
            
    def apply_workspace_edit_and_get_files(self, workspace_edit: dict) -> List[str]:
        """Apply a workspace edit and return the list of affected files."""
        affected_files = []
        
        try:
            # Handle both 'changes' and 'documentChanges' formats
            if "documentChanges" in workspace_edit:
                # Handle documentChanges format
                document_changes = workspace_edit["documentChanges"]
                
                for doc_change in document_changes:
                    file_uri = doc_change["textDocument"]["uri"]
                    text_edits = doc_change["edits"]
                    
                    # Convert URI back to file path
                    file_path = file_uri.replace("file://", "")
                    
                    # Apply the edits to this file
                    if apply_text_edits_to_file(file_path, text_edits):
                        affected_files.append(file_path)
                        
            elif "changes" in workspace_edit:
                # Handle changes format
                changes = workspace_edit["changes"]
                
                for file_uri, text_edits in changes.items():
                    # Convert URI back to file path
                    file_path = file_uri.replace("file://", "")
                    
                    # Apply the edits to this file
                    if apply_text_edits_to_file(file_path, text_edits):
                        affected_files.append(file_path)
            else:
                logger.error("Workspace edit contains neither 'changes' nor 'documentChanges'")
                
        except Exception as e:
            logger.error(f"Error applying workspace edit: {e}")
            
        return affected_files
            
    def open_document(self, file_path: str):
        """Open a document if not already opened."""
        if file_path in self.opened_documents:
            return
            
        with open(file_path, 'r', encoding='utf-8') as f:
            file_content = f.read()
        
        did_open_request = {
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": f"file://{Path(file_path).resolve()}",
                    "languageId": "rust",
                    "version": 1,
                    "text": file_content
                }
            }
        }
        
        self.send_request(did_open_request)
        self.opened_documents.add(file_path)
        
    def rename_symbol(self, file_path: str, line: int, col: int, new_name: str) -> bool:
        """Rename a symbol using LSP."""
        try:
            # Open the document if not already opened
            self.open_document(file_path)
            
            # First, prepare the rename to validate the range
            self.request_id += 1
            prepare_rename_request = {
                "jsonrpc": "2.0",
                "id": self.request_id,
                "method": "textDocument/prepareRename",
                "params": {
                    "textDocument": {"uri": f"file://{Path(file_path).resolve()}"},
                    "position": {"line": line, "character": col}
                }
            }
            
            self.send_request(prepare_rename_request)
            prepare_response = self.read_response()
            
            logger.debug(f"Received prepareRename response: {prepare_response}")
            
            if prepare_response.get("id") == self.request_id and "result" in prepare_response:
                # prepareRename succeeded, now perform the actual rename
                self.request_id += 1
                rename_request = {
                    "jsonrpc": "2.0",
                    "id": self.request_id,
                    "method": "textDocument/rename",
                    "params": {
                        "textDocument": {"uri": f"file://{Path(file_path).resolve()}"},
                        "position": {"line": line, "character": col},
                        "newName": new_name
                    }
                }
                
                self.send_request(rename_request)
                rename_response = self.read_response()
                
                logger.debug(f"Received rename response: {rename_response}")
                
                if rename_response.get("id") == self.request_id and "result" in rename_response:
                    workspace_edit = rename_response["result"]
                    if workspace_edit and ("changes" in workspace_edit or "documentChanges" in workspace_edit):
                        # Apply the workspace edit
                        affected_files = self.apply_workspace_edit_and_get_files(workspace_edit)
                        if affected_files:
                            # Update all affected documents in the LSP session
                            for affected_file in affected_files:
                                self.update_document(affected_file)
                            return True
                        else:
                            return False
                    else:
                        logger.warning(f"No changes in workspace edit for {file_path}:{line}:{col}")
                        return False
                else:
                    logger.error(f"Rename request for {new_name} failed at {file_path}:{line}:{col} - Response: {rename_response}")
                    return False
            else:
                logger.error(f"PrepareRename request failed at {file_path}:{line}:{col} - Response: {prepare_response}")
                return False
                
        except Exception as e:
            logger.error(f"Error during LSP rename: {e}")
            return False
            
    def close(self):
        """Close the LSP client."""
        if self.process:
            try:
                self.process.terminate()
                self.process.wait(timeout=5)
            except Exception:
                self.process.kill()

def run_rust_analyzer_rename_via_lsp(workspace_root: str, file_path: str, line: int, col: int, new_name: str) -> bool:
    """
    Legacy function - use RustAnalyzerLSPClient directly for better performance.
    """
    client = RustAnalyzerLSPClient(workspace_root)
    try:
        if not client.start():
            return False
        return client.rename_symbol(file_path, line, col, new_name)
    finally:
        client.close()

def apply_workspace_edit(workspace_edit: dict) -> bool:
    """Apply a workspace edit returned by rust-analyzer LSP."""
    try:
        # Handle both 'changes' and 'documentChanges' formats
        if "documentChanges" in workspace_edit:
            # Handle documentChanges format
            document_changes = workspace_edit["documentChanges"]
            
            for doc_change in document_changes:
                file_uri = doc_change["textDocument"]["uri"]
                text_edits = doc_change["edits"]
                
                # Convert URI back to file path
                file_path = file_uri.replace("file://", "")
                
                # Apply the edits to this file
                if not apply_text_edits_to_file(file_path, text_edits):
                    return False
                    
            return True
            
        elif "changes" in workspace_edit:
            # Handle changes format
            changes = workspace_edit["changes"]
            
            for file_uri, text_edits in changes.items():
                # Convert URI back to file path
                file_path = file_uri.replace("file://", "")
                
                # Apply the edits to this file
                if not apply_text_edits_to_file(file_path, text_edits):
                    return False
                    
            return True
        else:
            logger.error("Workspace edit contains neither 'changes' nor 'documentChanges'")
            return False
        
    except Exception as e:
        logger.error(f"Error applying workspace edit: {e}")
        return False

def apply_text_edits_to_file(file_path: str, text_edits: list) -> bool:
    """Apply a list of text edits to a specific file."""
    try:
        # Read the file
        with open(file_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        
        # Apply edits in reverse order to maintain positions
        text_edits.sort(key=lambda edit: (edit["range"]["start"]["line"], edit["range"]["start"]["character"]), reverse=True)
        
        for edit in text_edits:
            start_line = edit["range"]["start"]["line"]
            start_char = edit["range"]["start"]["character"]
            end_line = edit["range"]["end"]["line"]
            end_char = edit["range"]["end"]["character"]
            new_text = edit["newText"]
            
            # Apply the edit
            if start_line == end_line:
                # Single line edit
                line_content = lines[start_line]
                lines[start_line] = line_content[:start_char] + new_text + line_content[end_char:]
            else:
                # Multi-line edit - handle carefully
                # Replace from start position to end position
                start_line_content = lines[start_line]
                end_line_content = lines[end_line]
                
                # Create the new content
                new_content = start_line_content[:start_char] + new_text + end_line_content[end_char:]
                
                # Replace the lines
                lines[start_line:end_line + 1] = [new_content]
        
        # Write back the file
        with open(file_path, 'w', encoding='utf-8') as f:
            f.writelines(lines)
        
        logger.info(f"Applied workspace edit to {file_path}")
        return True
        
    except Exception as e:
        logger.error(f"Error applying text edits to {file_path}: {e}")
        return False



def fix_warnings_with_rust_analyzer(workspace_root: str, warnings: List[SnakeCaseWarning]) -> int:
    """
    Fix snake_case warnings using rust-analyzer's LSP rename functionality.
    This is safer than manual text replacement as it properly handles all references.
    """
    logger.info("Fixing warnings using rust-analyzer LSP rename...")
    
    total_fixed = 0
    
    # Sort warnings by file and position to avoid conflicts
    warnings.sort(key=lambda w: (w.file_path, w.line, w.col))
    
    # Use a single LSP client for all renames to improve performance
    client = RustAnalyzerLSPClient(workspace_root)
    try:
        if not client.start():
            logger.error("Failed to start rust-analyzer LSP client")
            return 0
        
        # Give rust-analyzer time to initialize and analyze the workspace
        import time
        time.sleep(1.0)
        
        for warning in warnings:
            if not warning.suggestion:
                logger.warning(f"No suggestion for warning: {warning.message}")
                continue
            
            if should_skip_warning(warning):
                logger.info(f"Skipping risky warning: {warning.file_path}:{warning.line}:{warning.col}")
                continue
            
            logger.info(f"Renaming identifier to '{warning.suggestion}' in {warning.file_path}:{warning.line}:{warning.col}")
            
            success = client.rename_symbol(
                warning.file_path,
                warning.end_line,
                warning.end_col,
                warning.suggestion
            )
            
            if success:
                total_fixed += 1
                logger.info(f"Successfully renamed to '{warning.suggestion}'")
            else:
                logger.warning(f"Failed to rename identifier at {warning.file_path}:{warning.line}:{warning.col}")
    
    finally:
        client.close()
    
    return total_fixed

def run_rust_analyzer_diagnostics(workspace_root: str) -> str:
    """Run rust-analyzer diagnostics and return the output."""
    logger.info("Running rust-analyzer diagnostics...")
    
    try:
        result = subprocess.run(
            ["rust-analyzer", "diagnostics", "."],
            cwd=workspace_root,
            capture_output=True,
            text=True,
            timeout=120  # 2 minutes timeout
        )
        
        if result.returncode != 0:
            logger.error(f"rust-analyzer diagnostics failed: {result.stderr}")
            return ""
        
        logger.info("rust-analyzer diagnostics completed successfully")
        return result.stdout
        
    except subprocess.TimeoutExpired:
        logger.error("rust-analyzer diagnostics timed out")
        return ""
    except Exception as e:
        logger.error(f"Failed to run rust-analyzer diagnostics: {e}")
        return ""

def main():
    """Main entry point."""
    workspace_root = "/home/alix_anneraud/Git/Xila/Core"
    
    # Try to open previous analysis results
    try:
        with open("./previous_analysis.txt", "r", encoding="utf-8") as f:
            output = f.read()
            logger.info("Loaded previous analysis results")
    except FileNotFoundError:
        output = run_rust_analyzer_diagnostics(workspace_root)
        if not output:
            logger.error("Failed to get rust-analyzer diagnostics")
            return
        open("./previous_analysis.txt", "w", encoding="utf-8").write(output)
        logger.info("Saved current analysis results for future reference")
    
    # Run rust-analyzer diagnostics
    

    
    # Parse warnings
    warnings = parse_rust_analyzer_output(output)
    
    if not warnings:
        logger.info("No snake_case warnings found")
        return
    
    logger.info(f"Found {len(warnings)} snake_case warnings")
    
    # Print warnings for review
    for warning in warnings:
        logger.info(f"Warning: {warning.file_path}:{warning.line}:{warning.col} - {warning.message}")
    
    # Ask for confirmation
    response = input(f"\nDo you want to fix {len(warnings)} warnings? [y/N]: ")
    if response.lower() != 'y':
        logger.info("Operation cancelled")
        return
    
    # Fix warnings
    logger.info("Fixing snake_case warnings...")
    fixed_count = fix_warnings_with_rust_analyzer(workspace_root, warnings)
    
    logger.info(f"Fixed {fixed_count} out of {len(warnings)} warnings")
    
    # Run diagnostics again to verify fixes
    if fixed_count > 0:
        logger.info("Running diagnostics again to verify fixes...")
        output = run_rust_analyzer_diagnostics(workspace_root)
        if output:
            remaining_warnings = parse_rust_analyzer_output(output)
            remaining_count = len(remaining_warnings)
            logger.info(f"Remaining snake_case warnings: {remaining_count}")
            
            if remaining_count > 0:
                logger.info("Some warnings may need manual review:")
                for warning in remaining_warnings[:10]:  # Show first 10
                    logger.info(f"  {warning.file_path}:{warning.line}:{warning.col} - {warning.message}")

if __name__ == "__main__":
    main()

