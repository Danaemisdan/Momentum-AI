import re
with open('src/app/page.tsx', 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Add isVisibleRef
content = content.replace('const [isVisible, setIsVisible] = useState(false);', 
                          'const [isVisible, setIsVisible] = useState(false);\n  const isVisibleRef = useRef(false);\n  useEffect(() => { isVisibleRef.current = isVisible; }, [isVisible]);')

# 2. Fix the stale closure references to isVisible inside the STT WebSocket handler
# Note: we don't want to replace ALL isVisible, just the ones in the STT handler. Let's do it safely.
content = content.replace('if (!isVisible && (lower.includes("momentum") || lower.includes("hey momentum")))', 
                          'if (!isVisibleRef.current && (lower.includes("momentum") || lower.includes("hey momentum")))')
content = content.replace('if (!isVisible) return;', 'if (!isVisibleRef.current) return;')
content = content.replace('if (!isVisible) {', 'if (!isVisibleRef.current) {')

with open('src/app/page.tsx', 'w', encoding='utf-8') as f:
    f.write(content)
