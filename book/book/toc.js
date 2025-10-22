// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded "><a href="getting-started/installation.html"><strong aria-hidden="true">1.</strong> Installation</a></li><li class="chapter-item expanded "><a href="getting-started/quick-start.html"><strong aria-hidden="true">2.</strong> Quick Start</a></li><li class="chapter-item expanded "><a href="getting-started/first-purification.html"><strong aria-hidden="true">3.</strong> Your First Purification</a></li><li class="chapter-item expanded "><a href="concepts/purification.html"><strong aria-hidden="true">4.</strong> What is Purification?</a></li><li class="chapter-item expanded "><a href="concepts/determinism.html"><strong aria-hidden="true">5.</strong> Determinism</a></li><li class="chapter-item expanded "><a href="concepts/idempotency.html"><strong aria-hidden="true">6.</strong> Idempotency</a></li><li class="chapter-item expanded "><a href="concepts/posix.html"><strong aria-hidden="true">7.</strong> POSIX Compliance</a></li><li class="chapter-item expanded "><a href="linting/security.html"><strong aria-hidden="true">8.</strong> Security Rules (SEC001-SEC008)</a></li><li class="chapter-item expanded "><a href="linting/determinism.html"><strong aria-hidden="true">9.</strong> Determinism Rules (DET001-DET003)</a></li><li class="chapter-item expanded "><a href="linting/idempotency.html"><strong aria-hidden="true">10.</strong> Idempotency Rules (IDEM001-IDEM003)</a></li><li class="chapter-item expanded "><a href="linting/custom-rules.html"><strong aria-hidden="true">11.</strong> Writing Custom Rules</a></li><li class="chapter-item expanded "><a href="config/overview.html"><strong aria-hidden="true">12.</strong> Overview</a></li><li class="chapter-item expanded "><a href="config/analyzing.html"><strong aria-hidden="true">13.</strong> Analyzing Config Files</a></li><li class="chapter-item expanded "><a href="config/purifying.html"><strong aria-hidden="true">14.</strong> Purifying .bashrc and .zshrc</a></li><li class="chapter-item expanded "><a href="config/rules/config-001.html"><strong aria-hidden="true">15.</strong> CONFIG-001: PATH Deduplication</a></li><li class="chapter-item expanded "><a href="config/rules/config-002.html"><strong aria-hidden="true">16.</strong> CONFIG-002: Quote Variables</a></li><li class="chapter-item expanded "><a href="makefile/overview.html"><strong aria-hidden="true">17.</strong> Makefile Overview</a></li><li class="chapter-item expanded "><a href="makefile/security.html"><strong aria-hidden="true">18.</strong> Makefile Security</a></li><li class="chapter-item expanded "><a href="makefile/best-practices.html"><strong aria-hidden="true">19.</strong> Makefile Best Practices</a></li><li class="chapter-item expanded "><a href="examples/bootstrap-installer.html"><strong aria-hidden="true">20.</strong> Bootstrap Installer</a></li><li class="chapter-item expanded "><a href="examples/deployment-script.html"><strong aria-hidden="true">21.</strong> Deployment Script</a></li><li class="chapter-item expanded "><a href="examples/config-management.html"><strong aria-hidden="true">22.</strong> Configuration Management</a></li><li class="chapter-item expanded "><a href="examples/cicd-pipeline.html"><strong aria-hidden="true">23.</strong> CI/CD Pipeline</a></li><li class="chapter-item expanded "><a href="advanced/ast-transformation.html"><strong aria-hidden="true">24.</strong> AST-Level Transformation</a></li><li class="chapter-item expanded "><a href="advanced/property-testing.html"><strong aria-hidden="true">25.</strong> Property Testing</a></li><li class="chapter-item expanded "><a href="advanced/mutation-testing.html"><strong aria-hidden="true">26.</strong> Mutation Testing</a></li><li class="chapter-item expanded "><a href="advanced/performance.html"><strong aria-hidden="true">27.</strong> Performance Optimization</a></li><li class="chapter-item expanded "><a href="reference/cli.html"><strong aria-hidden="true">28.</strong> CLI Commands</a></li><li class="chapter-item expanded "><a href="reference/configuration.html"><strong aria-hidden="true">29.</strong> Configuration</a></li><li class="chapter-item expanded "><a href="reference/exit-codes.html"><strong aria-hidden="true">30.</strong> Exit Codes</a></li><li class="chapter-item expanded "><a href="reference/rules.html"><strong aria-hidden="true">31.</strong> Linter Rules Reference</a></li><li class="chapter-item expanded "><a href="contributing/setup.html"><strong aria-hidden="true">32.</strong> Development Setup</a></li><li class="chapter-item expanded "><a href="contributing/extreme-tdd.html"><strong aria-hidden="true">33.</strong> EXTREME TDD</a></li><li class="chapter-item expanded "><a href="contributing/toyota-way.html"><strong aria-hidden="true">34.</strong> Toyota Way Principles</a></li><li class="chapter-item expanded "><a href="contributing/release.html"><strong aria-hidden="true">35.</strong> Release Process</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
