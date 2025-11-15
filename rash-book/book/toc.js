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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="title-page.html">The bashrs Programming Language</a></li><li class="chapter-item expanded affix "><a href="ch00-00-introduction.html">Introduction</a></li><li class="chapter-item expanded affix "><a href="test-status.html">📊 Test Status Dashboard</a></li><li class="chapter-item expanded "><a href="ch01-hello-shell-tdd.html"><strong aria-hidden="true">1.</strong> Chapter 1: Hello Shell Script</a></li><li class="chapter-item expanded "><a href="ch02-variables-tdd.html"><strong aria-hidden="true">2.</strong> Chapter 2: Variables and Assignment</a></li><li class="chapter-item expanded "><a href="ch03-functions-tdd.html"><strong aria-hidden="true">3.</strong> Chapter 3: Functions and Parameters</a></li><li class="chapter-item expanded "><a href="ch04-control-flow-tdd.html"><strong aria-hidden="true">4.</strong> Chapter 4: Control Flow</a></li><li class="chapter-item expanded "><a href="ch05-error-handling-tdd.html"><strong aria-hidden="true">5.</strong> Chapter 5: Error Handling</a></li><li class="chapter-item expanded "><a href="ch06-escaping-tdd.html"><strong aria-hidden="true">6.</strong> Chapter 6: String Escaping and Quoting</a></li><li class="chapter-item expanded "><a href="ch07-posix-compliance-tdd.html"><strong aria-hidden="true">7.</strong> Chapter 7: POSIX Compliance</a></li><li class="chapter-item expanded "><a href="ch08-shellcheck-tdd.html"><strong aria-hidden="true">8.</strong> Chapter 8: ShellCheck Validation</a></li><li class="chapter-item expanded "><a href="ch09-determinism-tdd.html"><strong aria-hidden="true">9.</strong> Chapter 9: Determinism and Idempotence</a></li><li class="chapter-item expanded "><a href="ch10-security-tdd.html"><strong aria-hidden="true">10.</strong> Chapter 10: Security and Injection Prevention</a></li><li class="chapter-item expanded "><a href="ch11-bootstrap-tdd.html"><strong aria-hidden="true">11.</strong> Chapter 11: Bootstrap Installers</a></li><li class="chapter-item expanded "><a href="ch12-config-tdd.html"><strong aria-hidden="true">12.</strong> Chapter 12: Configuration Management</a></li><li class="chapter-item expanded "><a href="ch13-verification-tdd.html"><strong aria-hidden="true">13.</strong> Chapter 13: Verification Levels</a></li><li class="chapter-item expanded "><a href="ch14-dialects-tdd.html"><strong aria-hidden="true">14.</strong> Chapter 14: Shell Dialects</a></li><li class="chapter-item expanded "><a href="ch15-ci-cd-tdd.html"><strong aria-hidden="true">15.</strong> Chapter 15: CI/CD Integration</a></li><li class="chapter-item expanded "><a href="ch16-mcp-server-tdd.html"><strong aria-hidden="true">16.</strong> Chapter 16: MCP Server Integration</a></li><li class="chapter-item expanded "><a href="ch17-testing-tdd.html"><strong aria-hidden="true">17.</strong> Chapter 17: Testing and Quality</a></li><li class="chapter-item expanded "><a href="ch21-makefile-linting-tdd.html"><strong aria-hidden="true">18.</strong> Chapter 21: Makefile and Shell Linting</a></li><li class="chapter-item expanded "><a href="ch18-limitations.html"><strong aria-hidden="true">19.</strong> Chapter 18: Known Limitations</a></li><li class="chapter-item expanded "><a href="ch19-best-practices.html"><strong aria-hidden="true">20.</strong> Chapter 19: Workarounds and Best Practices</a></li><li class="chapter-item expanded "><a href="ch20-roadmap.html"><strong aria-hidden="true">21.</strong> Chapter 20: Future Roadmap</a></li><li class="chapter-item expanded "><a href="appendix-a-installation.html"><strong aria-hidden="true">22.</strong> Appendix A: Installation Guide</a></li><li class="chapter-item expanded "><a href="appendix-b-glossary.html"><strong aria-hidden="true">23.</strong> Appendix B: Glossary</a></li><li class="chapter-item expanded "><a href="appendix-c-compatibility.html"><strong aria-hidden="true">24.</strong> Appendix C: Shell Compatibility Matrix</a></li><li class="chapter-item expanded "><a href="appendix-d-api.html"><strong aria-hidden="true">25.</strong> Appendix D: Complete API Reference</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString();
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
