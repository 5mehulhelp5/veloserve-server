// VeloServe Website JavaScript

document.addEventListener('DOMContentLoaded', () => {
    // Navigation scroll effect
    const nav = document.querySelector('.nav');
    
    window.addEventListener('scroll', () => {
        if (window.scrollY > 50) {
            nav.classList.add('scrolled');
        } else {
            nav.classList.remove('scrolled');
        }
    });

    // Tab switching for installation section
    const tabs = document.querySelectorAll('.tab');
    const tabContents = document.querySelectorAll('.tab-content');

    tabs.forEach(tab => {
        tab.addEventListener('click', () => {
            const targetId = tab.dataset.tab;
            
            // Update active tab
            tabs.forEach(t => t.classList.remove('active'));
            tab.classList.add('active');
            
            // Show corresponding content
            tabContents.forEach(content => {
                content.classList.remove('active');
                if (content.id === targetId) {
                    content.classList.add('active');
                }
            });
        });
    });

    // Smooth scroll for anchor links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            const target = document.querySelector(this.getAttribute('href'));
            if (target) {
                target.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
            }
        });
    });

    // Animate elements on scroll
    const observerOptions = {
        root: null,
        rootMargin: '0px',
        threshold: 0.1
    };

    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('visible');
            }
        });
    }, observerOptions);

    // Observe feature cards and other animated elements
    document.querySelectorAll('.feature-card, .mode-card, .step').forEach(el => {
        el.style.opacity = '0';
        el.style.transform = 'translateY(20px)';
        el.style.transition = 'opacity 0.5s ease, transform 0.5s ease';
        observer.observe(el);
    });

    // Add visible class handler
    const style = document.createElement('style');
    style.textContent = `
        .feature-card.visible,
        .mode-card.visible,
        .step.visible {
            opacity: 1 !important;
            transform: translateY(0) !important;
        }
    `;
    document.head.appendChild(style);

    // Stagger animation for grid items
    document.querySelectorAll('.features-grid .feature-card').forEach((card, index) => {
        card.style.transitionDelay = `${index * 0.1}s`;
    });

    // Copy code to clipboard on click
    document.querySelectorAll('.step-content pre, .mode-code').forEach(block => {
        block.style.cursor = 'pointer';
        block.title = 'Click to copy';
        
        block.addEventListener('click', () => {
            const code = block.textContent;
            navigator.clipboard.writeText(code).then(() => {
                // Show copied feedback
                const originalBg = block.style.background;
                block.style.background = 'rgba(34, 197, 94, 0.2)';
                setTimeout(() => {
                    block.style.background = originalBg;
                }, 300);
            });
        });
    });

    // Terminal typing effect for hero code
    const codeContent = document.querySelector('.code-content code');
    if (codeContent) {
        const originalHTML = codeContent.innerHTML;
        
        // Only run animation once when visible
        const codeObserver = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    // Animation could be added here
                    codeObserver.unobserve(entry.target);
                }
            });
        }, { threshold: 0.5 });
        
        codeObserver.observe(codeContent);
    }

    // Performance numbers animation
    const animateValue = (element, start, end, duration) => {
        let startTimestamp = null;
        const step = (timestamp) => {
            if (!startTimestamp) startTimestamp = timestamp;
            const progress = Math.min((timestamp - startTimestamp) / duration, 1);
            const value = Math.floor(progress * (end - start) + start);
            element.textContent = value.toLocaleString();
            if (progress < 1) {
                window.requestAnimationFrame(step);
            }
        };
        window.requestAnimationFrame(step);
    };

    // Observe stats for animation
    const statsObserver = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                const value = entry.target.querySelector('.stat-value');
                if (value && value.textContent.includes('10,000')) {
                    // Animate the requests/sec number
                    animateValue(value, 0, 10000, 1500);
                    value.textContent = '~10,000';
                }
                statsObserver.unobserve(entry.target);
            }
        });
    }, { threshold: 0.5 });

    document.querySelectorAll('.benchmark-row.highlight').forEach(row => {
        statsObserver.observe(row);
    });
});

// Add some console flair
console.log('%c⚡ VeloServe', 'font-size: 24px; font-weight: bold; color: #f97316;');
console.log('%cHigh-performance web server with embedded PHP', 'font-size: 14px; color: #a1a1aa;');
console.log('%chttps://github.com/veloserve/veloserve', 'font-size: 12px; color: #71717a;');

