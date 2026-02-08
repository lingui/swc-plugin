function collectMessage(path, props, ctx) {
  if (props.id === void 0) return;
  const node = path.node;
  const line = node.loc ? node.loc.start.line : void 0;
  const column = node.loc ? node.loc.start.column : void 0;
  ctx.opts.onMessageExtracted({
    id: props.id,
    message: props.message,
    context: props.context,
    comment: props.comment,
    placeholders: props.placeholders || {},
    origin: ctx.file.opts.filename ? [ctx.file.opts.filename, line, column] : void 0
  });
}
function getTextFromExpression(t, exp, hub, emitErrorOnVariable = true) {
  if (t.isStringLiteral(exp)) {
    return exp.value;
  }
  if (t.isBinaryExpression(exp)) {
    return getTextFromExpression(
      t,
      exp.left,
      hub,
      emitErrorOnVariable
    ) + getTextFromExpression(
      t,
      exp.right,
      hub,
      emitErrorOnVariable
    );
  }
  if (t.isTemplateLiteral(exp)) {
    if (exp?.quasis.length > 1) {
      console.warn(
        hub.buildError(
          exp,
          "Could not extract from template literal with expressions.",
          SyntaxError
        ).message
      );
      return "";
    }
    return exp.quasis[0]?.value?.cooked || "";
  }
  if (emitErrorOnVariable) {
    console.warn(
      hub.buildError(
        exp,
        "Only strings or template literals could be extracted.",
        SyntaxError
      ).message
    );
  }
  return "";
}
function getNodeSource(fileContents, node) {
  return fileContents.slice(node.start, node.end);
}
function valuesObjectExpressionToPlaceholdersRecord(t, exp, hub) {
  const props = {};
  exp.properties.forEach(({ key, value }, i) => {
    let name;
    if (t.isStringLiteral(key) || t.isNumericLiteral(key)) {
      name = key.value.toString();
    } else if (t.isIdentifier(key)) {
      name = key.name;
    } else {
      console.warn(
        hub.buildError(
          exp,
          `Could not extract values to placeholders. The key #${i} has unsupported syntax`,
          SyntaxError
        ).message
      );
    }
    if (name) {
      props[name] = getNodeSource(hub.getCode(), value);
    }
  });
  return props;
}
function extractFromObjectExpression(t, exp, hub) {
  const props = {};
  const textKeys = ["id", "message", "comment", "context"];
  exp.properties.forEach(({ key, value }, i) => {
    const name = key.name;
    if (name === "values" && t.isObjectExpression(value)) {
      props.placeholders = valuesObjectExpressionToPlaceholdersRecord(
        t,
        value,
        hub
      );
    } else if (textKeys.includes(name)) {
      props[name] = getTextFromExpression(
        t,
        value,
        hub
      );
    }
  });
  return props;
}
const I18N_OBJECT = "i18n";
function hasComment(node, comment) {
  return !!node.leadingComments?.some((comm) => comm.value.trim() === comment);
}
function hasIgnoreComment(node) {
  return hasComment(node, "lingui-extract-ignore");
}
function hasI18nComment(node) {
  return hasComment(node, "i18n");
}
function index({ types: t }) {
  function isTransComponent(path) {
    return path.isJSXElement() && path.get("openingElement").get("name").referencesImport("@lingui/react", "Trans");
  }
  const isI18nMethod = (node) => t.isMemberExpression(node) && (t.isIdentifier(node.object, { name: I18N_OBJECT }) || t.isMemberExpression(node.object) && t.isIdentifier(node.object.property, { name: I18N_OBJECT })) && (t.isIdentifier(node.property, { name: "_" }) || t.isIdentifier(node.property, { name: "t" }));
  const extractFromMessageDescriptor = (path, ctx) => {
    const props = extractFromObjectExpression(t, path.node, ctx.file.hub);
    if (!props.id) {
      console.warn(
        path.buildCodeFrameError("Missing message ID, skipping.").message
      );
      return;
    }
    collectMessage(path, props, ctx);
  };
  return {
    visitor: {
      // Extract translation from <Trans /> component.
      JSXElement(path, ctx) {
        const { node } = path;
        if (!isTransComponent(path)) return;
        const attrs = node.openingElement.attributes || [];
        if (attrs.find(
          (attr) => t.isJSXSpreadAttribute(attr) && hasI18nComment(attr.argument)
        )) {
          return;
        }
        const props = attrs.reduce((acc, item) => {
          if (t.isJSXSpreadAttribute(item)) {
            return acc;
          }
          const key = item.name.name;
          if (key === "id" || key === "message" || key === "comment" || key === "context") {
            if (t.isStringLiteral(item.value)) {
              acc[key] = item.value.value;
            } else if (t.isJSXExpressionContainer(item.value) && t.isStringLiteral(item.value.expression)) {
              acc[key] = item.value.expression.value;
            }
          }
          if (key === "values" && t.isJSXExpressionContainer(item.value) && t.isObjectExpression(item.value.expression)) {
            acc.placeholders = valuesObjectExpressionToPlaceholdersRecord(
              t,
              item.value.expression,
              ctx.file.hub
            );
          }
          return acc;
        }, {});
        if (!props.id) {
          const idProp = attrs.filter(
            (item) => t.isJSXAttribute(item) && item.name.name === "id"
          )[0];
          if (idProp === void 0 || t.isLiteral(props.id)) {
            console.warn(
              path.buildCodeFrameError("Missing message ID, skipping.").message
            );
          }
          return;
        }
        collectMessage(path, props, ctx);
      },
      CallExpression(path, ctx) {
        if ([path.node, path.parent].some((node) => hasIgnoreComment(node))) {
          return;
        }
        const firstArgument = path.get("arguments")[0];
        if (!isI18nMethod(path.node.callee)) {
          return;
        }
        if (hasI18nComment(firstArgument.node)) {
          return;
        }
        if (firstArgument.isObjectExpression()) {
          extractFromMessageDescriptor(firstArgument, ctx);
          return;
        } else {
          let props = {
            id: getTextFromExpression(
              t,
              firstArgument.node,
              ctx.file.hub,
              false
            )
          };
          if (!props.id) {
            return;
          }
          const secondArgument = path.node.arguments[1];
          if (secondArgument && t.isObjectExpression(secondArgument)) {
            props.placeholders = valuesObjectExpressionToPlaceholdersRecord(
              t,
              secondArgument,
              ctx.file.hub
            );
          }
          const msgDescArg = path.node.arguments[2];
          if (t.isObjectExpression(msgDescArg)) {
            props = {
              ...props,
              ...extractFromObjectExpression(t, msgDescArg, ctx.file.hub)
            };
          }
          collectMessage(path, props, ctx);
        }
      },
      StringLiteral(path, ctx) {
        if (!hasI18nComment(path.node)) {
          return;
        }
        const props = {
          id: path.node.value
        };
        if (!props.id) {
          console.warn(
            path.buildCodeFrameError("Empty StringLiteral, skipping.").message
          );
          return;
        }
        collectMessage(path, props, ctx);
      },
      // Extract message descriptors
      ObjectExpression(path, ctx) {
        if (!hasI18nComment(path.node)) return;
        extractFromMessageDescriptor(path, ctx);
      }
    }
  };
}

export { index as default };
