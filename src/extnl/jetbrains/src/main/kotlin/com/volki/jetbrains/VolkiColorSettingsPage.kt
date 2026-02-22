package com.volki.jetbrains

import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.fileTypes.SyntaxHighlighter
import com.intellij.openapi.options.colors.AttributesDescriptor
import com.intellij.openapi.options.colors.ColorDescriptor
import com.intellij.openapi.options.colors.ColorSettingsPage
import javax.swing.Icon

class VolkiColorSettingsPage : ColorSettingsPage {

    companion object {
        private val DESCRIPTORS = arrayOf(
            AttributesDescriptor("Keyword", VolkiSyntaxHighlighter.KEYWORD),
            AttributesDescriptor("Type", VolkiSyntaxHighlighter.TYPE),
            AttributesDescriptor("Tag name", VolkiSyntaxHighlighter.TAG_NAME),
            AttributesDescriptor("Tag bracket", VolkiSyntaxHighlighter.TAG_BRACKET),
            AttributesDescriptor("Attribute", VolkiSyntaxHighlighter.ATTRIBUTE),
            AttributesDescriptor("String", VolkiSyntaxHighlighter.STRING),
            AttributesDescriptor("Number", VolkiSyntaxHighlighter.NUMBER),
            AttributesDescriptor("Line comment", VolkiSyntaxHighlighter.LINE_COMMENT),
            AttributesDescriptor("Doc comment", VolkiSyntaxHighlighter.DOC_COMMENT),
            AttributesDescriptor("Block comment", VolkiSyntaxHighlighter.BLOCK_COMMENT),
            AttributesDescriptor("Braces", VolkiSyntaxHighlighter.BRACES),
            AttributesDescriptor("Parentheses", VolkiSyntaxHighlighter.PARENS),
            AttributesDescriptor("Brackets", VolkiSyntaxHighlighter.BRACKETS),
            AttributesDescriptor("Operator", VolkiSyntaxHighlighter.OPERATOR),
            AttributesDescriptor("Identifier", VolkiSyntaxHighlighter.IDENTIFIER),
            AttributesDescriptor("Function declaration", VolkiSyntaxHighlighter.FUNCTION_DECL),
            AttributesDescriptor("Function call", VolkiSyntaxHighlighter.FUNCTION_CALL),
            AttributesDescriptor("Method call", VolkiSyntaxHighlighter.METHOD_CALL),
            AttributesDescriptor("Variable", VolkiSyntaxHighlighter.VARIABLE),
            AttributesDescriptor("Type reference", VolkiSyntaxHighlighter.TYPE_REFERENCE),
            AttributesDescriptor("Return type", VolkiSyntaxHighlighter.RETURN_TYPE),
            AttributesDescriptor("Return arrow", VolkiSyntaxHighlighter.RETURN_ARROW),
            AttributesDescriptor("RSX//HTML tag name", VolkiSyntaxHighlighter.HTML_TAG_NAME),
            AttributesDescriptor("RSX//Custom component name", VolkiSyntaxHighlighter.CUSTOM_COMPONENT_NAME),
            AttributesDescriptor("Imports//Module path", VolkiSyntaxHighlighter.USE_PATH),
            AttributesDescriptor("Imports//Imported symbol", VolkiSyntaxHighlighter.USE_SYMBOL),
            AttributesDescriptor("Imports//Glob wildcard", VolkiSyntaxHighlighter.USE_GLOB),
            AttributesDescriptor("Expression//Ternary operators", VolkiSyntaxHighlighter.TERNARY_OPERATOR),
            AttributesDescriptor("Expression//Conditional and", VolkiSyntaxHighlighter.CONDITIONAL_AND),
        )

        private val ADDITIONAL_TAGS = mapOf(
            "htmlTag" to VolkiSyntaxHighlighter.HTML_TAG_NAME,
            "component" to VolkiSyntaxHighlighter.CUSTOM_COMPONENT_NAME,
            "usePath" to VolkiSyntaxHighlighter.USE_PATH,
            "useSymbol" to VolkiSyntaxHighlighter.USE_SYMBOL,
            "useGlob" to VolkiSyntaxHighlighter.USE_GLOB,
            "ternaryOp" to VolkiSyntaxHighlighter.TERNARY_OPERATOR,
            "condAnd" to VolkiSyntaxHighlighter.CONDITIONAL_AND,
            "fnDecl" to VolkiSyntaxHighlighter.FUNCTION_DECL,
            "fnCall" to VolkiSyntaxHighlighter.FUNCTION_CALL,
            "methodCall" to VolkiSyntaxHighlighter.METHOD_CALL,
            "varRef" to VolkiSyntaxHighlighter.VARIABLE,
            "typeRef" to VolkiSyntaxHighlighter.TYPE_REFERENCE,
            "returnType" to VolkiSyntaxHighlighter.RETURN_TYPE,
            "returnArrow" to VolkiSyntaxHighlighter.RETURN_ARROW,
        )
    }

    override fun getIcon(): Icon = VolkiIcons.FILE

    override fun getHighlighter(): SyntaxHighlighter = VolkiSyntaxHighlighter()

    override fun getDemoText(): String = """
        //! Module documentation comment
        use <usePath>crate</usePath>::<usePath>core</usePath>::{<useSymbol>Html</useSymbol>, <useSymbol>Fragment</useSymbol>, <useSymbol>Client</useSymbol>};
        use <usePath>crate</usePath>::<usePath>libs</usePath>::<usePath>web</usePath>::<usePath>prelude</usePath>::<useGlob>*</useGlob>;

        // A full-featured page component
        pub fn <fnDecl>HomePage</fnDecl>(<varRef>req</varRef>: &<typeRef>Request</typeRef>) <returnArrow>-></returnArrow> <returnType>Html</returnType> {
            let <varRef>title</varRef>: &<typeRef>str</typeRef> = "Welcome to Volki";
            let <varRef>count</varRef>: <typeRef>i32</typeRef> = 42;
            let <varRef>is_active</varRef>: <typeRef>bool</typeRef> = true;
            let base_url = r#"https://example.com/api"#;

            /* Multi-line
               block comment */
            <<htmlTag>div</htmlTag> class="container" style="padding: 1rem;">
                <<htmlTag>nav</htmlTag> class="top-nav">
                    <<htmlTag>a</htmlTag> href="/">Home</<htmlTag>a</htmlTag>>
                    <<htmlTag>a</htmlTag> href="/about">About</<htmlTag>a</htmlTag>>
                </<htmlTag>nav</htmlTag>>

                <<htmlTag>header</htmlTag> class="hero">
                    <<htmlTag>h1</htmlTag>>{<varRef>title</varRef>}</<htmlTag>h1</htmlTag>>
                    <<htmlTag>p</htmlTag>>{"Visited " + <varRef>count</varRef>.<methodCall>to_string</methodCall>() + " times."}</<htmlTag>p</htmlTag>>
                </<htmlTag>header</htmlTag>>

                <<htmlTag>section</htmlTag> class="content">
                    {is_active <condAnd>&&</condAnd> <<htmlTag>p</htmlTag>>"active user"</<htmlTag>p</htmlTag>>}
                    {is_active <ternaryOp>?</ternaryOp> <<htmlTag>span</htmlTag>>"online"</<htmlTag>span</htmlTag>> <ternaryOp>:</ternaryOp> <<htmlTag>span</htmlTag>>"offline"</<htmlTag>span</htmlTag>>}
                    <<htmlTag>img</htmlTag> src="/banner.png" alt="Banner" />
                    <<htmlTag>input</htmlTag> type="text" placeholder="Search..." />

                    <<component>Button</component> onclick={<varRef>handle_click</varRef>} label="Click me" />
                    <<component>NavBar</component> items={nav_items} active={is_active} />

                    <<htmlTag>table</htmlTag> class="data-table">
                        <<htmlTag>thead</htmlTag>>
                            <<htmlTag>tr</htmlTag>><<htmlTag>th</htmlTag>>Name</<htmlTag>th</htmlTag>><<htmlTag>th</htmlTag>>License</<htmlTag>th</htmlTag>></<htmlTag>tr</htmlTag>>
                        </<htmlTag>thead</htmlTag>>
                        <<htmlTag>tbody</htmlTag>>
                    {<varRef>rows</varRef>.<methodCall>iter</methodCall>().<methodCall>map</methodCall>(|<varRef>row</varRef>| {
                                <<htmlTag>tr</htmlTag>>
                                    <<htmlTag>td</htmlTag>>{<varRef>row</varRef>.name}</<htmlTag>td</htmlTag>>
                                    <<htmlTag>td</htmlTag>>{<varRef>row</varRef>.license}</<htmlTag>td</htmlTag>>
                                </<htmlTag>tr</htmlTag>>
                            })}
                        </<htmlTag>tbody</htmlTag>>
                    </<htmlTag>table</htmlTag>>

                    <<htmlTag>ul</htmlTag>>
                    {<varRef>items</varRef>.<methodCall>iter</methodCall>().<methodCall>map</methodCall>(|<varRef>item</varRef>| {
                            <<htmlTag>li</htmlTag> class="item">{<varRef>item</varRef>.name}</<htmlTag>li</htmlTag>>
                        })}
                    </<htmlTag>ul</htmlTag>>
                </<htmlTag>section</htmlTag>>

                <<htmlTag>footer</htmlTag> class="site-footer">
                    <<htmlTag>span</htmlTag> style="color: gray;">
                        <<htmlTag>small</htmlTag>>{"v" + <varRef>VERSION</varRef>.<methodCall>to_string</methodCall>()}</<htmlTag>small</htmlTag>>
                    </<htmlTag>span</htmlTag>>
                </<htmlTag>footer</htmlTag>>
            </<htmlTag>div</htmlTag>>
        }

        struct AppState {
            count: u32,
            name: <typeRef>String</typeRef>,
            items: <typeRef>Vec</typeRef><<typeRef>Option</typeRef><<typeRef>String</typeRef>>>,
            metadata: <typeRef>HashMap</typeRef><<typeRef>String</typeRef>, <typeRef>u64</typeRef>>,
        }

        impl AppState {
            pub fn <fnDecl>new</fnDecl>(<varRef>name</varRef>: &<typeRef>str</typeRef>) <returnArrow>-></returnArrow> <returnType>Self</returnType> {
                let <varRef>scores</varRef>: [<typeRef>f64</typeRef>; 3] = [0.5, 1.0, 0xff_u8 as <typeRef>f64</typeRef>];
                Self { count: 0, name: <varRef>name</varRef>.<methodCall>into</methodCall>(), items: <typeRef>Vec</typeRef>::<fnCall>new</fnCall>(), metadata: <typeRef>HashMap</typeRef>::<fnCall>new</fnCall>() }
            }
        }
    """.trimIndent()

    override fun getAdditionalHighlightingTagToDescriptorMap(): Map<String, TextAttributesKey> = ADDITIONAL_TAGS

    override fun getAttributeDescriptors(): Array<AttributesDescriptor> = DESCRIPTORS

    override fun getColorDescriptors(): Array<ColorDescriptor> = ColorDescriptor.EMPTY_ARRAY

    override fun getDisplayName(): String = "Volki"
}
