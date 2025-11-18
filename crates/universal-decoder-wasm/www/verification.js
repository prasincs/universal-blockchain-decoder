/**
 * Verification Dashboard Renderer
 * Renders the Verus formal verification status in the WASM demo
 */

export async function initVerificationDashboard(wasm) {
    try {
        const verificationData = wasm.get_verification_status();
        renderDashboard(verificationData);
    } catch (error) {
        console.error('Failed to load verification data:', error);
        document.getElementById('verification-dashboard').innerHTML = `
            <div class="error">
                <p>⚠️ Failed to load verification data</p>
            </div>
        `;
    }
}

function renderDashboard(data) {
    const container = document.getElementById('verification-dashboard');

    const html = `
        <div class="verification-container">
            <div class="verification-header">
                <h2>🔬 Verus Formal Verification Status</h2>
                <p class="subtitle">Mathematical proofs of correctness for critical security properties</p>
            </div>

            <!-- Overall Progress -->
            <div class="verification-section">
                <h3>📊 Overall Progress</h3>
                <div class="progress-overview">
                    <div class="progress-bar-container">
                        <div class="progress-bar" style="width: ${data.overall.percentage}%">
                            <span class="progress-text">${data.overall.percentage}%</span>
                        </div>
                    </div>
                    <p class="progress-label">
                        ${data.overall.completed} / ${data.overall.total} Verification Conditions (VCs)
                    </p>
                </div>
            </div>

            <!-- Security Properties Proven -->
            <div class="verification-section">
                <h3>💎 Security Properties Proven</h3>
                <div class="properties-grid">
                    ${data.properties.map(prop => `
                        <div class="property-card ${prop.critical ? 'critical' : ''}">
                            <div class="property-header">
                                <span class="property-icon">✅</span>
                                <span class="property-name">${prop.name}</span>
                                ${prop.critical ? '<span class="badge-critical">CRITICAL</span>' : ''}
                            </div>
                            <p class="property-description">${prop.description}</p>
                            <p class="property-impact"><strong>Impact:</strong> ${prop.impact}</p>
                            <p class="property-vcs"><em>VCs: ${prop.vcs}</em></p>
                        </div>
                    `).join('')}
                </div>
            </div>

            <!-- Phase Breakdown -->
            <div class="verification-section">
                <h3>📋 Verification Phases</h3>
                ${data.phases.map((phase, index) => `
                    <div class="phase-card ${phase.status}">
                        <div class="phase-header">
                            <h4>${phase.name}</h4>
                            <span class="badge-${phase.status}">${phase.status.toUpperCase()}</span>
                        </div>
                        <div class="phase-progress">
                            <div class="progress-bar-container small">
                                <div class="progress-bar" style="width: ${(phase.vcs / phase.total * 100)}%"></div>
                            </div>
                            <span class="phase-stats">${phase.vcs} / ${phase.total} VCs</span>
                        </div>
                        ${phase.modules && phase.modules.length > 0 ? `
                            <div class="modules-list">
                                ${phase.modules.map(module => `
                                    <div class="module-item ${module.status}">
                                        <span class="module-icon">${module.status === 'complete' ? '✅' : module.status === 'todo' ? '○' : '◐'}</span>
                                        <span class="module-name">${module.name}</span>
                                        <span class="module-vcs">${module.vcs || 0}/${module.total} VCs</span>
                                        ${module.critical ? '<span class="badge-critical-sm">CRITICAL</span>' : ''}
                                    </div>
                                    ${module.items && module.status === 'complete' ? `
                                        <ul class="module-items">
                                            ${module.items.map(item => `<li>✓ ${item}</li>`).join('')}
                                        </ul>
                                    ` : ''}
                                `).join('')}
                            </div>
                        ` : ''}
                    </div>
                `).join('')}
            </div>

            <!-- Timeline -->
            <div class="verification-section">
                <h3>⏱️ Timeline</h3>
                <div class="timeline-stats">
                    <div class="timeline-item">
                        <span class="timeline-label">Completed:</span>
                        <span class="timeline-value">${data.timeline.completed_weeks} weeks</span>
                    </div>
                    <div class="timeline-item">
                        <span class="timeline-label">Total Estimated:</span>
                        <span class="timeline-value">${data.timeline.total_weeks} weeks (~${Math.round(data.timeline.total_weeks / 4)} months)</span>
                    </div>
                    <div class="timeline-item">
                        <span class="timeline-label">Remaining:</span>
                        <span class="timeline-value">${data.timeline.remaining_weeks} weeks (~${Math.round(data.timeline.remaining_weeks / 4)} months)</span>
                    </div>
                </div>
            </div>

            <!-- Footer -->
            <div class="verification-footer">
                <p>
                    <strong>What this means:</strong> Verus formal verification provides mathematical proofs
                    that our code is correct—not just "tested" but <em>provably correct</em> under formal logic.
                </p>
                <p>
                    Learn more:
                    <a href="https://github.com/prasincs/universal-blockchain-decoder/blob/main/docs/VERUS_WHAT_IT_PROVES.md" target="_blank">What Verus Proves</a> |
                    <a href="https://github.com/prasincs/universal-blockchain-decoder/blob/main/docs/VERIFICATION_TARGETS.md" target="_blank">Verification Targets</a>
                </p>
            </div>
        </div>
    `;

    container.innerHTML = html;
}

// Add CSS dynamically (can also be in style.css)
export function addVerificationStyles() {
    const style = document.createElement('style');
    style.textContent = `
        .verification-container {
            padding: 20px;
            max-width: 1200px;
            margin: 0 auto;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }

        .verification-header {
            text-align: center;
            margin-bottom: 30px;
        }

        .verification-header h2 {
            margin: 0 0 10px 0;
            color: #1e40af;
        }

        .subtitle {
            color: #6b7280;
            font-size: 0.9rem;
        }

        .verification-section {
            margin-bottom: 30px;
            background: #f9fafb;
            border-radius: 8px;
            padding: 20px;
        }

        .verification-section h3 {
            margin-top: 0;
            color: #374151;
        }

        .progress-bar-container {
            background: #e5e7eb;
            border-radius: 10px;
            height: 30px;
            position: relative;
            overflow: hidden;
        }

        .progress-bar-container.small {
            height: 20px;
        }

        .progress-bar {
            background: linear-gradient(90deg, #10b981, #059669);
            height: 100%;
            border-radius: 10px;
            transition: width 0.3s ease;
            display: flex;
            align-items: center;
            justify-content: center;
        }

        .progress-text {
            color: white;
            font-weight: bold;
            font-size: 0.9rem;
        }

        .progress-label {
            text-align: center;
            margin-top: 10px;
            font-size: 0.9rem;
            color: #6b7280;
        }

        .properties-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 15px;
        }

        .property-card {
            background: white;
            border: 2px solid #10b981;
            border-radius: 8px;
            padding: 15px;
        }

        .property-card.critical {
            border-color: #ef4444;
        }

        .property-header {
            display: flex;
            align-items: center;
            gap: 10px;
            margin-bottom: 10px;
        }

        .property-icon {
            font-size: 1.2rem;
        }

        .property-name {
            font-weight: bold;
            color: #1f2937;
        }

        .badge-critical {
            background: #ef4444;
            color: white;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 0.7rem;
            font-weight: bold;
        }

        .badge-critical-sm {
            background: #ef4444;
            color: white;
            padding: 1px 6px;
            border-radius: 3px;
            font-size: 0.65rem;
            font-weight: bold;
        }

        .property-description {
            font-size: 0.9rem;
            color: #4b5563;
            margin: 5px 0;
        }

        .property-impact {
            font-size: 0.85rem;
            color: #6b7280;
            margin: 5px 0;
        }

        .property-vcs {
            font-size: 0.8rem;
            color: #9ca3af;
            margin-top: 5px;
        }

        .phase-card {
            background: white;
            border-radius: 8px;
            padding: 15px;
            margin-bottom: 15px;
            border-left: 4px solid #6b7280;
        }

        .phase-card.complete {
            border-left-color: #10b981;
        }

        .phase-card.planned {
            border-left-color: #f59e0b;
        }

        .phase-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 10px;
        }

        .phase-header h4 {
            margin: 0;
            color: #1f2937;
        }

        .badge-complete {
            background: #10b981;
            color: white;
            padding: 4px 12px;
            border-radius: 4px;
            font-size: 0.75rem;
            font-weight: bold;
        }

        .badge-planned {
            background: #f59e0b;
            color: white;
            padding: 4px 12px;
            border-radius: 4px;
            font-size: 0.75rem;
            font-weight: bold;
        }

        .phase-progress {
            display: flex;
            align-items: center;
            gap: 10px;
            margin-bottom: 10px;
        }

        .phase-stats {
            font-size: 0.9rem;
            color: #6b7280;
            white-space: nowrap;
        }

        .modules-list {
            margin-top: 10px;
        }

        .module-item {
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 8px;
            background: #f9fafb;
            border-radius: 4px;
            margin-bottom: 5px;
            font-size: 0.9rem;
        }

        .module-item.complete {
            background: #d1fae5;
        }

        .module-item.todo {
            background: #f3f4f6;
            color: #9ca3af;
        }

        .module-icon {
            font-size: 1rem;
        }

        .module-name {
            flex: 1;
            font-weight: 500;
        }

        .module-vcs {
            color: #6b7280;
            font-size: 0.85rem;
        }

        .module-items {
            margin: 5px 0 5px 30px;
            padding: 0;
            list-style: none;
        }

        .module-items li {
            font-size: 0.85rem;
            color: #6b7280;
            padding: 2px 0;
        }

        .timeline-stats {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
        }

        .timeline-item {
            background: white;
            padding: 15px;
            border-radius: 6px;
            text-align: center;
        }

        .timeline-label {
            display: block;
            font-size: 0.85rem;
            color: #6b7280;
            margin-bottom: 5px;
        }

        .timeline-value {
            display: block;
            font-size: 1.1rem;
            font-weight: bold;
            color: #1f2937;
        }

        .verification-footer {
            margin-top: 30px;
            padding: 20px;
            background: #eff6ff;
            border-radius: 8px;
            text-align: center;
        }

        .verification-footer p {
            margin: 10px 0;
            color: #1e40af;
        }

        .verification-footer a {
            color: #2563eb;
            text-decoration: underline;
        }

        .verification-footer a:hover {
            color: #1d4ed8;
        }
    `;
    document.head.appendChild(style);
}
