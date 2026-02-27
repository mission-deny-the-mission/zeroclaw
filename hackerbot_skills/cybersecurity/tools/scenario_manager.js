#!/usr/bin/env node

/**
 * Scenario Manager Tool for Cybersecurity Training
 * 
 * Manages cybersecurity training scenarios including:
 * - Scenario navigation (list, next, previous, goto)
 * - State tracking per user
 * - Quiz answer validation
 * - Progress tracking
 * 
 * Usage:
 *   scenario_manager list --user <username>
 *   scenario_manager goto --user <username> --index <n>
 *   scenario_manager next --user <username>
 *   scenario_manager previous --user <username>
 *   scenario_manager submit-answer --user <username> --answer <text>
 *   scenario_manager get-current --user <username>
 */

const fs = require('fs');
const path = require('path');

// State storage location
const STATE_DIR = path.join(process.env.HOME || process.env.USERPROFILE, '.zeroclaw', 'state');
const STATE_FILE = path.join(STATE_DIR, 'scenario_state.json');

// Ensure state directory exists
if (!fs.existsSync(STATE_DIR)) {
    fs.mkdirSync(STATE_DIR, { recursive: true });
}

// Default scenarios
const DEFAULT_SCENARIOS = [
    {
        index: 0,
        title: "Network Reconnaissance with nmap",
        prompt: "Let's explore network reconnaissance using nmap. What are the common nmap scanning techniques, and how would you use nmap to discover open ports and services on a target network?",
        quiz: {
            question: "Which nmap option performs a TCP SYN scan (half-open scan)?",
            answer: "-sS",
            accepted_variants: [
                "sS",
                "-sS SYN scan",
                "SYN scan",
                "half-open scan"
            ],
            correct_response: "Correct! The -sS option performs a TCP SYN scan (half-open scan), which doesn't complete the TCP three-way handshake, making it stealthier than a full connect scan.",
            explanation: "TCP SYN scans send only the initial SYN packet and wait for a response. If a SYN-ACK is received, the port is open. If RST is received, the port is closed. This is faster and less detectable than full TCP connects."
        },
        demonstration: {
            command: "nmap -sS -p 1-1000 target.example.com",
            expected_output_pattern: "Starting Nmap",
            learning_points: [
                "-sS: TCP SYN scan (stealth scan)",
                "-p 1-1000: Scan first 1000 ports",
                "Results show open, closed, or filtered ports"
            ]
        }
    },
    {
        index: 1,
        title: "Traffic Analysis with tcpdump",
        prompt: "Now let's examine network traffic analysis using tcpdump. How would you use tcpdump to capture and analyze network traffic, specifically focusing on suspicious patterns that might indicate malicious activity?",
        quiz: {
            question: "What tcpdump filter expression would capture all HTTP traffic (port 80) to or from IP 192.168.1.100?",
            answer: "host 192.168.1.100 and port 80",
            accepted_variants: [
                "port 80 and host 192.168.1.100",
                "tcpdump host 192.168.1.100 and port 80",
                "-i any host 192.168.1.100 and port 80"
            ],
            correct_response: "Correct! The filter 'host 192.168.1.100 and port 80' captures HTTP traffic to/from that specific IP. Tcpdump's filtering is essential for focused traffic analysis.",
            explanation: "The 'host' keyword specifies an IP address, 'and' combines filters, and 'port' specifies the port number. This filter captures packets where BOTH conditions are true."
        },
        demonstration: {
            command: "tcpdump -i any -n 'host 192.168.1.100 and port 80' -c 10",
            expected_output_pattern: "listening on",
            learning_points: [
                "-i any: Capture on all interfaces",
                "-n: Don't resolve hostnames (faster)",
                "-c 10: Capture 10 packets then stop",
                "Filter expressions reduce noise"
            ]
        }
    },
    {
        index: 2,
        title: "Firewall Configuration with iptables",
        prompt: "Let's analyze firewall configuration using iptables. How would you configure iptables to create rules that block common attack patterns while allowing legitimate traffic?",
        quiz: {
            question: "What iptables command would block all incoming SSH connections (port 22) from IP 10.0.0.50?",
            answer: "iptables -A INPUT -s 10.0.0.50 -p tcp --dport 22 -j DROP",
            accepted_variants: [
                "iptables -A INPUT -s 10.0.0.50 -p tcp --dport 22 -j REJECT",
                "iptables -A INPUT -s 10.0.0.50 --dport 22 -j DROP",
                "iptables -A INPUT -s 10.0.0.50 -p tcp --dport ssh -j DROP"
            ],
            correct_response: "Correct! This command appends a rule to the INPUT chain that drops TCP packets destined for port 22 from the specified IP. This is a fundamental firewall blocking technique.",
            explanation: "The rule structure is: -A (append to chain) INPUT (incoming traffic) -s (source IP) 10.0.0.50 -p tcp (protocol) --dport 22 (destination port) -j DROP (action to drop packets)."
        },
        demonstration: {
            command: "iptables -L -n -v --line-numbers",
            expected_output_pattern: "Chain INPUT",
            learning_points: [
                "-L: List all rules",
                "-n: Numeric output (no DNS resolution)",
                "-v: Verbose (show packet counts)",
                "--line-numbers: Show rule numbers for easy reference"
            ]
        }
    },
    {
        index: 3,
        title: "Secure Communication with SSH",
        prompt: "Finally, let's explore secure communication using SSH. What are the security best practices for SSH configuration, and how would you harden an SSH server against common attacks?",
        quiz: {
            question: "What SSH configuration directive would disable root login and force key-based authentication only?",
            answer: "PermitRootLogin no and PasswordAuthentication no",
            accepted_variants: [
                "PermitRootLogin no, PasswordAuthentication no",
                "disable root login and password auth",
                "PermitRootLogin no; PasswordAuthentication no"
            ],
            correct_response: "Correct! Disabling root login (PermitRootLogin no) and forcing key-based authentication (PasswordAuthentication no) are fundamental SSH hardening steps that significantly reduce brute force attack risk.",
            explanation: "These settings are in /etc/ssh/sshd_config. 'PermitRootLogin no' prevents direct root access, forcing attackers to compromise a user account first. 'PasswordAuthentication no' requires SSH keys, eliminating password-based attacks."
        },
        demonstration: {
            command: "sshd -T | grep -E 'permitrootlogin|passwordauthentication'",
            expected_output_pattern: "permitrootlogin",
            learning_points: [
                "sshd -T: Show effective configuration",
                "grep filters for specific settings",
                "Verify changes before restarting sshd",
                "Always test SSH config in a separate session"
            ]
        }
    }
];

// Load state from file
function loadState() {
    try {
        if (fs.existsSync(STATE_FILE)) {
            return JSON.parse(fs.readFileSync(STATE_FILE, 'utf8'));
        }
    } catch (error) {
        console.error('Warning: Could not load state file:', error.message);
    }
    return { users: {} };
}

// Save state to file
function saveState(state) {
    try {
        fs.writeFileSync(STATE_FILE, JSON.stringify(state, null, 2));
    } catch (error) {
        console.error('Error saving state file:', error.message);
    }
}

// Get or create user state
function getUserState(username) {
    const state = loadState();
    if (!state.users[username]) {
        state.users[username] = {
            currentScenarioIndex: 0,
            completedScenarios: [],
            quizAttempts: [],
            lastActivity: new Date().toISOString()
        };
        saveState(state);
    }
    return state.users[username];
}

// List all scenarios
function listScenarios(username) {
    const userState = getUserState(username);
    console.log('Available cybersecurity training scenarios:');
    console.log('');
    
    DEFAULT_SCENARIOS.forEach((scenario, index) => {
        const current = index === userState.currentScenarioIndex ? ' [CURRENT]' : '';
        const completed = userState.completedScenarios.includes(index) ? ' ✓' : '';
        console.log(`${index + 1}. ${scenario.title}${current}${completed}`);
    });
    
    console.log('');
    console.log(`Progress: ${userState.completedScenarios.length}/${DEFAULT_SCENARIOS.length} completed`);
}

// Go to specific scenario
function gotoScenario(username, index) {
    if (index < 1 || index > DEFAULT_SCENARIOS.length) {
        console.error(`Error: Invalid scenario index. Must be between 1 and ${DEFAULT_SCENARIOS.length}`);
        process.exit(1);
    }
    
    const state = loadState();
    state.users[username].currentScenarioIndex = index - 1;
    state.users[username].lastActivity = new Date().toISOString();
    saveState(state);
    
    const scenario = DEFAULT_SCENARIOS[index - 1];
    console.log(`Jumped to scenario ${index}: ${scenario.title}`);
    console.log('');
    console.log(scenario.prompt);
}

// Move to next scenario
function nextScenario(username) {
    const state = loadState();
    const userState = state.users[username];
    
    if (userState.currentScenarioIndex >= DEFAULT_SCENARIOS.length - 1) {
        console.log("You're already at the final scenario. Great job completing all scenarios!");
        return;
    }
    
    userState.currentScenarioIndex++;
    userState.lastActivity = new Date().toISOString();
    saveState(state);
    
    const scenario = DEFAULT_SCENARIOS[userState.currentScenarioIndex];
    console.log(`Moving to scenario ${userState.currentScenarioIndex + 1}: ${scenario.title}`);
    console.log('');
    console.log(scenario.prompt);
}

// Move to previous scenario
function previousScenario(username) {
    const state = loadState();
    const userState = state.users[username];
    
    if (userState.currentScenarioIndex <= 0) {
        console.log("You're already at the first scenario.");
        return;
    }
    
    userState.currentScenarioIndex--;
    userState.lastActivity = new Date().toISOString();
    saveState(state);
    
    const scenario = DEFAULT_SCENARIOS[userState.currentScenarioIndex];
    console.log(`Going back to scenario ${userState.currentScenarioIndex + 1}: ${scenario.title}`);
    console.log('');
    console.log(scenario.prompt);
}

// Get current scenario
function getCurrentScenario(username) {
    const userState = getUserState(username);
    const scenario = DEFAULT_SCENARIOS[userState.currentScenarioIndex];
    
    console.log(`Current scenario: ${userState.currentScenarioIndex + 1}. ${scenario.title}`);
    console.log('');
    console.log(scenario.prompt);
    
    if (scenario.quiz) {
        console.log('');
        console.log('Quiz available. Submit your answer with: scenario_manager submit-answer --user <username> --answer "<your answer>"');
    }
}

// Submit quiz answer
function submitAnswer(username, answer) {
    const userState = getUserState(username);
    const scenario = DEFAULT_SCENARIOS[userState.currentScenarioIndex];
    
    if (!scenario.quiz) {
        console.log("No quiz available for this scenario.");
        return;
    }
    
    const normalizedAnswer = answer.toLowerCase().trim();
    const normalizedCorrect = scenario.quiz.answer.toLowerCase().trim();
    const isCorrect = normalizedAnswer === normalizedCorrect || 
                      scenario.quiz.accepted_variants.some(v => normalizedAnswer.includes(v.toLowerCase()));
    
    // Record attempt
    userState.quizAttempts.push({
        scenarioIndex: userState.currentScenarioIndex,
        answer: answer,
        correct: isCorrect,
        timestamp: new Date().toISOString()
    });
    
    const state = loadState();
    state.users[username] = userState;
    saveState(state);
    
    if (isCorrect) {
        console.log('✓ Correct!');
        console.log('');
        console.log(scenario.quiz.correct_response);
        console.log('');
        console.log('Explanation:');
        console.log(scenario.quiz.explanation);
        
        // Mark scenario as completed if not already
        if (!userState.completedScenarios.includes(userState.currentScenarioIndex)) {
            userState.completedScenarios.push(userState.currentScenarioIndex);
            const state = loadState();
            state.users[username] = userState;
            saveState(state);
        }
        
        console.log('');
        console.log('Scenario completed! Use "scenario_manager next --user <username>" to continue.');
    } else {
        console.log('✗ Not quite right. Try again or ask for a hint.');
        console.log('');
        console.log('Hint: Think about the key concepts we discussed.');
    }
}

// Get scenario demonstration command
function getDemonstration(username) {
    const userState = getUserState(username);
    const scenario = DEFAULT_SCENARIOS[userState.currentScenarioIndex];
    
    if (!scenario.demonstration) {
        console.log("No demonstration available for this scenario.");
        return;
    }
    
    console.log('Demonstration command:');
    console.log(scenario.demonstration.command);
    console.log('');
    console.log('Learning points:');
    scenario.demonstration.learning_points.forEach(point => {
        console.log(`  - ${point}`);
    });
}

// Show progress
function showProgress(username) {
    const userState = getUserState(username);
    
    console.log(`Progress for ${username}:`);
    console.log('');
    console.log(`Current scenario: ${userState.currentScenarioIndex + 1}/${DEFAULT_SCENARIOS.length}`);
    console.log(`Completed scenarios: ${userState.completedScenarios.length}/${DEFAULT_SCENARIOS.length}`);
    console.log(`Quiz attempts: ${userState.quizAttempts.length}`);
    
    if (userState.quizAttempts.length > 0) {
        const correct = userState.quizAttempts.filter(a => a.correct).length;
        const accuracy = Math.round((correct / userState.quizAttempts.length) * 100);
        console.log(`Quiz accuracy: ${accuracy}% (${correct}/${userState.quizAttempts.length} correct)`);
    }
    
    console.log(`Last activity: ${userState.lastActivity}`);
}

// Main command parser
const args = process.argv.slice(2);
const command = args[0];

function parseArgs(args) {
    const parsed = {};
    for (let i = 1; i < args.length; i++) {
        if (args[i] === '--user' && args[i + 1]) {
            parsed.user = args[++i];
        } else if (args[i] === '--index' && args[i + 1]) {
            parsed.index = parseInt(args[++i], 10);
        } else if (args[i] === '--answer' && args[i + 1]) {
            parsed.answer = args[++i];
        }
    }
    return parsed;
}

const parsedArgs = parseArgs(args);

if (!command) {
    console.log('Usage:');
    console.log('  scenario_manager <command> --user <username> [options]');
    console.log('');
    console.log('Commands:');
    console.log('  list                 List all scenarios');
    console.log('  get-current          Get current scenario');
    console.log('  goto --index <n>     Go to scenario n');
    console.log('  next                 Go to next scenario');
    console.log('  previous             Go to previous scenario');
    console.log('  submit-answer --answer <text>  Submit quiz answer');
    console.log('  get-demonstration    Get demonstration command');
    console.log('  progress             Show progress');
    process.exit(1);
}

if (!parsedArgs.user && command !== 'help') {
    console.error('Error: --user <username> is required');
    process.exit(1);
}

switch (command) {
    case 'list':
        listScenarios(parsedArgs.user);
        break;
    case 'get-current':
        getCurrentScenario(parsedArgs.user);
        break;
    case 'goto':
        if (!parsedArgs.index) {
            console.error('Error: --index <n> is required');
            process.exit(1);
        }
        gotoScenario(parsedArgs.user, parsedArgs.index);
        break;
    case 'next':
        nextScenario(parsedArgs.user);
        break;
    case 'previous':
        previousScenario(parsedArgs.user);
        break;
    case 'submit-answer':
        if (!parsedArgs.answer) {
            console.error('Error: --answer <text> is required');
            process.exit(1);
        }
        submitAnswer(parsedArgs.user, parsedArgs.answer);
        break;
    case 'get-demonstration':
        getDemonstration(parsedArgs.user);
        break;
    case 'progress':
        showProgress(parsedArgs.user);
        break;
    default:
        console.error(`Unknown command: ${command}`);
        process.exit(1);
}
