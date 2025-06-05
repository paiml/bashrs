use crate::models::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn intersects(&self, other: &Rect) -> bool {
        !(self.x + self.width <= other.x
            || other.x + other.width <= self.x
            || self.y + self.height <= other.y
            || other.y + other.height <= self.y)
    }
}

#[derive(Debug, Clone)]
pub enum Widget {
    SourceEditor,
    TranspileOutput,
    DiagnosticPanel,
}

#[derive(Debug)]
pub enum DrawCommand {
    Clear(Rect),
    DrawText {
        rect: Rect,
        text: String,
        style: u32, // Style ID
    },
    DrawBorder {
        rect: Rect,
        style: u32,
    },
}

/// Buffer for differential rendering
pub struct Buffer {
    #[cfg(feature = "playground")]
    cells: Vec<Vec<char>>,
    width: u16,
    height: u16,
}

impl Buffer {
    pub fn new(width: u16, height: u16) -> Self {
        #[cfg(feature = "playground")]
        {
            Self {
                cells: vec![vec![' '; width as usize]; height as usize],
                width,
                height,
            }
        }

        #[cfg(not(feature = "playground"))]
        {
            Self { width, height }
        }
    }

    pub fn clear_region(&mut self, _region: &Rect) {
        #[cfg(feature = "playground")]
        {
            let start_x = _region.x.min(self.width) as usize;
            let end_x = (_region.x + _region.width).min(self.width) as usize;
            let start_y = _region.y.min(self.height) as usize;
            let end_y = (_region.y + _region.height).min(self.height) as usize;

            for y in start_y..end_y {
                for x in start_x..end_x {
                    if y < self.cells.len() && x < self.cells[y].len() {
                        self.cells[y][x] = ' ';
                    }
                }
            }
        }
    }

    pub fn diff(&self, _other: &Buffer) -> Vec<DrawCommand> {
        // TODO: Implement efficient diff algorithm
        vec![]
    }
}

pub struct LayoutCache {
    cache: std::collections::HashMap<Rect, Widget>,
}

impl Default for LayoutCache {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutCache {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    pub fn get_widget_at(&self, rect: &Rect) -> Widget {
        self.cache
            .get(rect)
            .cloned()
            .unwrap_or(Widget::SourceEditor)
    }

    pub fn set_widget(&mut self, rect: Rect, widget: Widget) {
        self.cache.insert(rect, widget);
    }
}

/// Zero-copy rendering pipeline with differential updates
pub struct RenderPipeline {
    front_buffer: Buffer,
    back_buffer: Buffer,
    damage_regions: Vec<Rect>,
    layout_cache: LayoutCache,
    full_refresh: bool,
}

impl RenderPipeline {
    pub fn new() -> Result<Self> {
        // Start with minimal size, will be resized on first frame
        let front_buffer = Buffer::new(80, 24);
        let back_buffer = Buffer::new(80, 24);

        Ok(Self {
            front_buffer,
            back_buffer,
            damage_regions: Vec::new(),
            layout_cache: LayoutCache::new(),
            full_refresh: true,
        })
    }

    pub fn mark_full_refresh(&mut self) {
        self.full_refresh = true;
        self.damage_regions.clear();
    }

    pub fn add_damage_region(&mut self, rect: Rect) {
        // Avoid duplicate regions
        if !self.damage_regions.iter().any(|r| r.intersects(&rect)) {
            self.damage_regions.push(rect);
        }
    }

    pub fn render_frame(&mut self, _state: &PlaygroundState) -> Result<Vec<DrawCommand>> {
        let mut commands = Vec::new();

        if self.full_refresh {
            // Clear entire back buffer
            self.back_buffer.clear_region(&Rect {
                x: 0,
                y: 0,
                width: 80, // TODO: Get actual terminal size
                height: 24,
            });

            // Add full screen as damage region
            self.damage_regions.push(Rect {
                x: 0,
                y: 0,
                width: 80,
                height: 24,
            });

            self.full_refresh = false;
        }

        // Clear damage regions in back buffer
        for region in &self.damage_regions {
            self.back_buffer.clear_region(region);
        }

        // Render only damaged regions
        for region in &self.damage_regions {
            match self.layout_cache.get_widget_at(region) {
                Widget::SourceEditor => {
                    self.render_source_lines(region, _state, &mut commands)?;
                }
                Widget::TranspileOutput => {
                    self.render_shell_output(region, _state, &mut commands)?;
                }
                Widget::DiagnosticPanel => {
                    self.render_diagnostics(region, _state, &mut commands)?;
                }
            }
        }

        // Compute diff between buffers
        let diff_commands = self.front_buffer.diff(&self.back_buffer);
        commands.extend(diff_commands);

        // Swap buffers
        std::mem::swap(&mut self.front_buffer, &mut self.back_buffer);
        self.damage_regions.clear();

        Ok(commands)
    }

    fn render_source_lines(
        &self,
        region: &Rect,
        _state: &PlaygroundState,
        commands: &mut Vec<DrawCommand>,
    ) -> Result<()> {
        // TODO: Implement source editor rendering
        commands.push(DrawCommand::DrawText {
            rect: region.clone(),
            text: "// Source editor placeholder".to_string(),
            style: 0,
        });

        Ok(())
    }

    fn render_shell_output(
        &self,
        region: &Rect,
        _state: &PlaygroundState,
        commands: &mut Vec<DrawCommand>,
    ) -> Result<()> {
        // TODO: Implement shell output rendering
        commands.push(DrawCommand::DrawText {
            rect: region.clone(),
            text: "#!/bin/sh\necho \"Hello, World!\"".to_string(),
            style: 1,
        });

        Ok(())
    }

    fn render_diagnostics(
        &self,
        region: &Rect,
        _state: &PlaygroundState,
        commands: &mut Vec<DrawCommand>,
    ) -> Result<()> {
        // TODO: Implement diagnostics rendering
        commands.push(DrawCommand::DrawText {
            rect: region.clone(),
            text: "No diagnostics".to_string(),
            style: 2,
        });

        Ok(())
    }
}

// Forward declaration - actual struct defined in system.rs
pub struct PlaygroundState;
